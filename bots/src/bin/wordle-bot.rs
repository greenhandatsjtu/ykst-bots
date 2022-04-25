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
struct ParseActionError;

impl Display for ParseActionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Unable to parse to the action.")
    }
}

impl Error for ParseActionError {
    fn description(&self) -> &str {
        "Unable to parse to the action."
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

// https://qubyte.codes/blog/parsing-input-from-stdin-to-structures-in-rust
impl FromStr for Action {
    type Err = ParseActionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let iter = s.split_whitespace();
        let tokens = iter.collect::<Vec<&str>>();
        let action: Action;
        match tokens[0] {
            "/start" => action = Action::Start,
            "/guess" => {
                if tokens.len() > 1 {
                    let guess = tokens[1];
                    if !(guess.len() == 5 && guess.chars().all(char::is_alphabetic)) {
                        return Err(ParseActionError);
                    }
                    action = Action::Guess(guess.to_lowercase().to_string());
                } else {
                    return Err(ParseActionError);
                }
            }
            _ => action = Action::Nop
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
        if replies.posts.len() == 0 {
            continue;
        }
        for post in replies.posts {
            floor = post.floor;
            let content = post.content.as_str();
            // println!("{} {}", floor, content);
            // check post time
            if !checked {
                if let Some(model) = &post.model {
                    let post_time = &model.created_at.as_ref().unwrap();
                    let since_the_epoch = now.duration_since(time::UNIX_EPOCH)?;
                    // println!("{} {}", since_the_epoch.as_secs(), post_time.seconds);
                    if since_the_epoch.as_secs() as i64 >= post_time.seconds {
                        continue;
                    } else {
                        checked = true;
                    }
                }
            }
            let res = content.parse::<Action>();
            if res.is_err() {
                continue;
            }
            let action = res.unwrap();
            match action {
                Action::Start => {
                    if game.is_none() {
                        game = Some(Game::from_day(rand::thread_rng().gen(), cl_wordle::words::ORIGINAL));
                        bot.reply_to_thread(thread_id, String::from("Wordle started")).await?;
                    } else {
                        bot.reply_to_thread(thread_id, String::from("Wordle already started")).await?;
                    }
                }
                Action::Guess(guess) => {
                    if let Some(g) = game.as_mut() {
                        let mut reply: String;
                        let result = g.guess(guess.as_str());
                        if result.is_err() {
                            reply = String::from("Invalid word");
                        } else {
                            reply = format!("{}", result.unwrap());
                        }
                        if let Some(_end) = g.game_over() {
                            reply = g.solution().to_string();
                            for gu in g.guesses() {
                                write!(reply, "\n\n{}", gu.1)?;
                            }
                            game = None;
                        }
                        bot.reply_to_thread(thread_id, reply).await?;
                    } else {
                        bot.reply_to_thread(thread_id, String::from("Please start game first")).await?;
                    }
                }
                _ => {}
            }
        }
    }
}
