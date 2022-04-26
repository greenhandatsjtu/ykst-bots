use ykst_client;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("API_URL")?;
    let token = settings.get_string("TREEHOLE_TOKEN")?;
    let identity = settings.get_string("IDENTITY_CODE")?;

    let mut client = ykst_client::Client::new(api_url, token, identity).await?;

    let _user = client.get_profile().await?;

    let threads = client.get_user_threads().await?;
    println!("{:?}", threads.threads[0]);

    let title = String::from("test title");
    let content = String::from("test content");
    let thread = client.create_thread(ykst_client::Category::Main, title, content, None).await?;
    println!("{:#?}", thread);

    let content = String::from("test reply");
    let post = client.reply_to_thread(5227, content).await?;
    println!("{:#?}", post);

    Ok(())
}
