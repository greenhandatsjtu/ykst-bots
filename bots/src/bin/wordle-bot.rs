#[macro_use]
extern crate log;

use std::error::Error;
use std::{fmt, time};
use std::fmt::{Display, Write, Formatter};
use std::str::FromStr;
use std::thread::sleep;
use ykst_client;
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
            ParseActionError::EmptyWordError => write!(f, "❌  猜测单词为空，请输入5个字母组成的英文单词"),
            ParseActionError::UnsupportedActionError(action) => write!(f, "❌  `{}` 为不支持的动作，请输入`/start`或`/guess guess`", action)
        }
    }
}

impl Error for ParseActionError {}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Nop => write!(f, "nop"),
            Action::Guess(guess) => write!(f, "/guess {}", guess),
            Action::Start => write!(f, "/start")
        }
    }
}

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
    env_logger::init();
    info!("read settings");
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("API_URL")?;
    let token = settings.get_string("TREEHOLE_TOKEN")?;
    let identity = settings.get_string("IDENTITY_CODE")?;
    let thread_id: u64 = settings.get_string("THREAD_ID")?.parse()?;

    info!("connect to treehole");
    let mut client = ykst_client::Client::new(api_url, token, identity).await?;

    // let now = time::SystemTime::now();
    // let mut checked = false; // flag to indicate if bot has checked time
    let mut game: Option<Game> = None;
    let mut guesses: Vec<String> = vec![String::new(); 6];
    let mut n_try = 0;

    let thread = client.get_thread(thread_id).await?;
    let mut floor = thread.reply_count;
    info!("thread floor: {}", floor);

    info!("start loop");
    loop {
        sleep(time::Duration::from_secs(2));
        let replies;
        // info!("get threads replies");
        if let Ok(res) = client.get_thread_replies(thread_id, floor, 19).await {
            replies = res;
        } else {
            // info!("no new replies");
            continue;
        }
        for post in replies.posts {
            floor = post.floor; // update post floor
            let content = post.content.as_str();
            // println!("{} {}", floor, content);
            let post_id: u64;
            if let Some(model) = &post.model {
                post_id = model.id;
                // check post time
                // if !checked {
                //     if let Some(post_time) = &model.created_at.as_ref() {
                //         let since_the_epoch = now.duration_since(time::UNIX_EPOCH)?;
                //         // println!("{} {}", since_the_epoch.as_secs(), post_time.seconds);
                //         if since_the_epoch.as_secs() as i64 >= post_time.seconds {
                //             continue;
                //         } else {
                //             info!("new replies from now on");
                //             checked = true;
                //         }
                //     } else {
                //         warn!("mode.created_at is none");
                //         continue;
                //     }
                // }
            } else {
                warn!("post.model is none");
                continue;
            }
            // skip bot replies
            // if post.identity_code == bot.identity {
            //     continue;
            // }
            let res = content.parse::<Action>();
            if res.is_err() {
                // failed to parse action
                info!("failed to parse action");
                let _ = client.reply_to_thread(thread_id, format!("{}", res.err().unwrap())).await;
                continue;
            }
            let action = res.unwrap();
            info!("floor: {} action: {}", floor, action);
            match action {
                Action::Start => {
                    if game.is_none() {
                        // start game
                        let g = Game::from_day(rand::thread_rng().gen(), cl_wordle::words::NYTIMES);
                        info!("game started, answer: {}", g.solution());
                        game = Some(g);
                        n_try = 0; // reset count of try
                        let _ = client.reply_to_thread(thread_id, String::from("🚀  Wordle 游戏开始，请输入`/guess guess`猜词，谜底为5位单词，一共6次机会，首先猜对的用户获胜。\n\n每次反馈的方格都会显示三种颜色，表示猜测和答案的接近程度：\n\n🟩代表该字母正确，对应字母**加粗**\n\n🟨代表谜底里有该字母但位置不对\n\n⬛代表谜底没有该字母，对应字母~~删除~~")).await;
                    } else {
                        // game already started
                        info!("game already started");
                        let _ = client.reply_to_thread(thread_id, String::from("❌  游戏已经开始，请输入`/guess guess`猜词")).await;
                    }
                }
                Action::Guess(guess) => {
                    if let Some(g) = game.as_mut() {
                        let mut reply: String = String::new();
                        // validate guess
                        let result = g.guess(guess.as_str());
                        if result.is_err() {
                            info!("invalid guess");
                            reply = format!("❌  `{}` 为无效词汇，请确保单词为5个英文字母组成且有效", guess);
                            let _ = client.reply_to_thread(thread_id, reply).await;
                            continue; // continue to avoid panic when calling game_over() when there's no guess
                        } else {
                            let matches = result.unwrap();
                            guesses[n_try].clear();
                            for i in 0..5 {
                                let ch = guess.chars().nth(i).unwrap();
                                match &matches.0[i] {
                                    cl_wordle::Match::Exact => write!(guesses[n_try], " **{}**", ch)?,
                                    cl_wordle::Match::Close => write!(guesses[n_try], " {}", ch)?,
                                    cl_wordle::Match::Wrong => write!(guesses[n_try], " ~~{}~~", ch)?
                                }
                            }
                            write!(guesses[n_try], "    @{}", post.identity_code)?;
                            // show all history guesses
                            let mut i = 0;
                            for gu in g.guesses() {
                                write!(reply, "\n\n{} {}", gu.1, guesses[i])?;
                                i += 1;
                            }
                            n_try += 1; // increment count of try
                        }
                        if let Some(end) = g.game_over() {
                            // reply.clear();
                            reply = format!("## {} {}/6{}", g.solution(), n_try, reply);
                            n_try = 0;
                            if end.is_win() {
                                info!("game ends, win");
                                write!(reply, "\n\n 恭喜{}，小鱼干奉上🎉", post.identity_code)?;
                                let _ = client.appreciate_post(post_id, 1).await;
                            } else {
                                info!("game ends, lose");
                                write!(reply, "\n\n 游戏结束，再接再厉💪")?;
                            }
                            game = None;
                        }
                        let _ = client.reply_to_thread(thread_id, reply).await;
                    } else {
                        // game not started
                        info!("game not started");
                        let _ = client.reply_to_thread(thread_id, String::from("❌  游戏还未开始，请回复`/start`以开始游戏")).await;
                    }
                }
                _ => {}
            }
        }
    }
}
