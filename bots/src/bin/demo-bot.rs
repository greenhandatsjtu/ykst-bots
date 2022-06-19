use config::Config;
use ykst_client;
use ykst_client::model::RateType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("API_URL")?;
    let token = settings.get_string("TREEHOLE_TOKEN")?;
    let identity = settings.get_string("IDENTITY_CODE")?;

    let mut client = ykst_client::Client::new(api_url, token, identity, Some(5)).await?;

    // get profile (user info)
    let _user = client.get_profile().await?;

    // check in
    let _ = client.checkin().await;

    // get user created threads
    let threads = client.get_user_threads().await?;
    println!("{:?}", threads.threads[0]);

    // create a new thread
    let title = String::from("test title");
    let content = String::from("test content");
    let thread = client
        .create_thread(ykst_client::Category::Main, title, content, None)
        .await?;
    println!("{:#?}", thread);

    let thread_id = thread.model.unwrap().id;

    // like thread
    let _ = client.rate_thread(thread_id, RateType::Like).await?;

    // reply to thread
    let content = String::from("test reply");
    let post = client.reply_to_thread(thread_id, content).await?;
    println!("{:#?}", post);

    // like post
    let post_id = post.model.unwrap().id;
    let _ = client.rate_post(post_id, RateType::Like).await?;

    Ok(())
}
