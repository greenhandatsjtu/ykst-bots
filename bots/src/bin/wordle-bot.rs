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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Alphabet(pub [cl_wordle::Match; 26]);

impl Display for Alphabet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut res: fmt::Result = Ok(());
        for (i, m) in self.0.iter().enumerate() {
            if i % 7 == 0 {
                res = write!(f, "\n\n");
                if res.is_err() {
                    return res;
                }
            }
            let ch = (i as u8 + 'A' as u8) as char;
            match m {
                cl_wordle::Match::Wrong => res = write!(f, "~~{}~~ ", ch),
                cl_wordle::Match::Close => res = write!(f, "{} ", ch),
                cl_wordle::Match::Exact => res = write!(f, "***{}*** ", ch)
            }
            if res.is_err() {
                return res;
            }
        }
        res
    }
}

struct Wordle {
    game: Game,
    feedbacks: Vec<String>,
    alphabet: Alphabet,
}

impl Wordle {
    fn new() -> Self {
        let game = Game::from_day(rand::thread_rng().gen(), cl_wordle::words::NYTIMES);
        Wordle {
            game,
            feedbacks: vec![],
            alphabet: Alphabet { 0: [cl_wordle::Match::Close; 26] },
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

    let mut wordle: Option<Wordle> = None;

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
            } else {
                warn!("post.model is none");
                continue;
            }
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
                    if wordle.is_none() {
                        // start game
                        let w = Wordle::new();
                        info!("game started, answer: {}", w.game.solution());
                        wordle = Some(w);
                        let _ = client.reply_to_thread(thread_id, String::from("🚀  Wordle 游戏开始，请输入`/guess guess`猜词，谜底为5位单词，一共6次机会，首先猜对的用户获胜。\n\n每次反馈都包括猜测的历史记录和字母表，历史记录的方格会显示三种颜色，表示猜测和答案的接近程度：\n\n+ 🟩代表该字母正确，对应字母***斜体加粗***\n\n+ 🟨代表谜底里有该字母但位置不对\n\n+ ⬛代表谜底没有该字母，对应字母~~删除~~\n\n字母表中***斜体加粗***代表谜底里有该字母，~~删除~~代表谜底没有该字母")).await;
                    } else {
                        // game already started
                        info!("game already started");
                        let _ = client.reply_to_thread(thread_id, String::from("❌  游戏已经开始，请输入`/guess guess`猜词")).await;
                    }
                }
                Action::Guess(guess) => {
                    if let Some(w) = wordle.as_mut() {
                        let mut reply: String = String::new();
                        // validate guess
                        let result = w.game.guess(guess.as_str());
                        if result.is_err() {
                            info!("invalid guess");
                            reply = format!("❌  `{}` 为无效词汇，请确保单词为5个英文字母组成且有效", guess);
                            let _ = client.reply_to_thread(thread_id, reply).await;
                            continue; // continue to avoid panic when calling game_over() when there's no guess
                        } else {
                            let matches = result.unwrap();
                            let mut feedback = String::new();
                            for (i, ch) in guess.chars().enumerate() {
                                match &matches.0[i] {
                                    cl_wordle::Match::Exact => {
                                        write!(feedback, " ***{}***", ch)?;
                                        w.alphabet.0[ch as usize - 'a' as usize] = cl_wordle::Match::Exact;
                                    }
                                    cl_wordle::Match::Close => {
                                        write!(feedback, " {}", ch)?;
                                        w.alphabet.0[ch as usize - 'a' as usize] = cl_wordle::Match::Exact;
                                    }
                                    cl_wordle::Match::Wrong => {
                                        write!(feedback, " ~~{}~~", ch)?;
                                        if w.alphabet.0[ch as usize - 'a' as usize] == cl_wordle::Match::Close {
                                            // When the answer is leant, and the guess is erase, the first e is Close and second `e` is Wrong
                                            w.alphabet.0[ch as usize - 'a' as usize] = cl_wordle::Match::Wrong;
                                        }
                                    }
                                }
                            }
                            write!(feedback, "    @{}", post.identity_code)?;
                            w.feedbacks.push(feedback); // add feedback to feedbacks
                            // show all history guesses
                            for (i, gu) in w.game.guesses().enumerate() {
                                write!(reply, "\n\n{} {}", gu.1, w.feedbacks[i])?;
                            }
                        }
                        if let Some(end) = w.game.game_over() {
                            reply = format!("## {} {}/6{}", w.game.solution(), w.feedbacks.len(), reply);
                            if end.is_win() {
                                info!("game ends, win");
                                write!(reply, "\n\n 恭喜{}，小鱼干奉上🎉", post.identity_code)?;
                                let _ = client.appreciate_post(post_id, 1).await;
                            } else {
                                info!("game ends, lose");
                                write!(reply, "\n\n 游戏结束，再接再厉💪")?;
                            }
                            wordle = None;
                        } else {
                            // print alphabet
                            write!(reply, "\n\n___\n\n {}", w.alphabet)?;
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
