use ykst_bot;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("API_URL")?;
    let token = settings.get_string("TREEHOLE_TOKEN")?;
    let identity = settings.get_string("IDENTITY_CODE")?;

    let mut bot = ykst_bot::Bot::new(api_url, token, identity).await?;

    let content = String::from("test reply");
    let post = bot.reply_to_thread(5227, content).await?;
    println!("{:#?}", post);

    Ok(())
}
