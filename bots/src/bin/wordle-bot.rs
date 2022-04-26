use std::error::Error;
use std::{fmt, time};
use std::fmt::{Display, Write, Formatter};
use std::str::FromStr;
use std::thread::sleep;
use ykst_bot;
use cl_wordle::{game::Game};
use config::Config;
use rand::Rng;

enum Action {
    Nop,
    Start,
    Guess(String),
}

#[derive(Debug, Clone)]
enum ParseActionError {
    InvalidWordError(String),
    EmptyWordError,
    UnsupportedActionError(String),
}

// https://fettblog.eu/rust-enums-wrapping-errors/
impl Display for ParseActionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseActionError::InvalidWordError(word) => write!(f, "❌  `{}` 为无效词汇，请确保单词为5个英文字母组成", word),
            ParseActionError::EmptyWordError => write!(f, "❌  猜测单词为空"),
            ParseActionError::UnsupportedActionError(action) => write!(f, "❌  `{}` 为不支持的动作", action)
        }
    }
}

impl Error for ParseActionError {}

// https://qubyte.codes/blog/parsing-input-from-stdin-to-structures-in-rust
impl FromStr for Action {
    type Err = ParseActionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let iter = s.trim().split_whitespace(); // trim spaces and split by spaces
        let tokens = iter.collect::<Vec<&str>>();
        let action: Action;
        if tokens[0].starts_with("/") {
            match tokens[0] {
                "/start" => action = Action::Start,
                "/guess" => {
                    if tokens.len() > 1 {
                        let guess = tokens[1];
                        if !(guess.len() == 5 && guess.chars().all(char::is_alphabetic)) {
                            return Err(ParseActionError::InvalidWordError(guess.to_string()));
                        }
                        action = Action::Guess(guess.to_lowercase().to_string());
                    } else {
                        return Err(ParseActionError::EmptyWordError);
                    }
                }
                _ => return Err(ParseActionError::UnsupportedActionError(tokens[0].to_string()))
            }
        } else {
            action = Action::Nop
        }
        Ok(action)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("API_URL")?;
    let token = settings.get_string("TREEHOLE_TOKEN")?;
    let identity = settings.get_string("IDENTITY_CODE")?;
    let thread_id: u64 = settings.get_string("THREAD_ID")?.parse()?;

    let mut bot = ykst_bot::Bot::new(api_url, token, identity).await?;

    let now = time::SystemTime::now();
    let mut checked = false;
    let mut floor = 0;
    let mut game: Option<Game> = None;
    loop {
        sleep(time::Duration::from_secs(1));
        let replies = bot.get_thread_replies(thread_id, floor, 10).await?;
        for post in replies.posts {
            floor = post.floor; // update post floor
            let content = post.content.as_str();
            // println!("{} {}", floor, content);
            let post_id: u64;
            if let Some(model) = &post.model {
                post_id = model.id;
                // check post time
                if !checked {
                    let post_time = &model.created_at.as_ref().unwrap();
                    let since_the_epoch = now.duration_since(time::UNIX_EPOCH)?;
                    // println!("{} {}", since_the_epoch.as_secs(), post_time.seconds);
                    if since_the_epoch.as_secs() as i64 >= post_time.seconds {
                        continue;
                    } else {
                        checked = true;
                    }
                }
            } else {
                continue;
            }
            // skip bot replies
            // if post.identity_code == bot.identity {
            //     continue;
            // }
            let res = content.parse::<Action>();
            if res.is_err() {
                let _ = bot.reply_to_post(thread_id, Some(post_id), format!("{}", res.err().unwrap())).await;
                continue;
            }
            let action = res.unwrap();
            match action {
                Action::Start => {
                    if game.is_none() {
                        game = Some(Game::from_day(rand::thread_rng().gen(), cl_wordle::words::NYTIMES));
                        let _ = bot.reply_to_post(thread_id, Some(post_id), String::from("Wordle started")).await;
                    } else {
                        let _ = bot.reply_to_post(thread_id, Some(post_id), String::from("Wordle already started")).await;
                    }
                }
                Action::Guess(guess) => {
                    if let Some(g) = game.as_mut() {
                        let mut reply: String;
                        let result = g.guess(guess.as_str());
                        println!("{}", g.solution());
                        if result.is_err() {
                            reply = String::from("Invalid word");
                        } else {
                            reply = format!("{}", result.unwrap());
                        }
                        if let Some(end) = g.game_over() {
                            reply = g.solution().to_string();
                            for gu in g.guesses() {
                                write!(reply, "\n\n{}", gu.1)?;
                            }
                            if end.is_win() {
                                write!(reply, "\n\n You win!")?;
                                if let Some(model) = &post.model {
                                    let _ = bot.appreciate_post(model.id, 1).await;
                                }
                            } else {
                                write!(reply, "\n\n You lose.")?;
                            }
                            game = None;
                        }
                        let _ = bot.reply_to_post(thread_id, Some(post_id), reply).await;
                    } else {
                        let _ = bot.reply_to_post(thread_id, Some(post_id), String::from("Please start game first")).await;
                    }
                }
                _ => {}
            }
        }
    }
}
