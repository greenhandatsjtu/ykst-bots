use config::Config;
use reqwest;
use std::env;
use ykst_client::model::tree_hole_client::TreeHoleClient;
use ykst_client::model::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("AUTH_API_URL")?;
    let redirect_url = settings.get_string("AUTH_REDIRECT_URL")?;

    let mut client = TreeHoleClient::connect(api_url).await?;

    println!("Fetching oauth config...");
    let request = tonic::Request::new(OAuthConfigRequest {
        channel: OAuthLoginChannel::LoginWithJAccount as i32,
        source: LoginSource::Web as i32,
    });
    let config = client.get_o_auth_config(request).await?.into_inner();

    println!("Getting code...");
    let http_client = reqwest::Client::new();
    let url = format!(
        "{}?response_type=code&client_id={}&scope={}&redirect_uri={}",
        config.authorize_url, config.client_id, config.scopes[0], redirect_url
    );
    println!("{}", url);
    let cookie = env::var("JACCOUNT_COOKIE").expect("JACCOUNT_COOKIE env not presented");
    let response = http_client
        .get(url)
        .header(reqwest::header::COOKIE, cookie)
        .send()
        .await?;
    let url: String = response.url().to_string();
    let code = url.split("code=").collect::<Vec<&str>>()[1].to_string();
    println!("Code: {}", code);

    println!("Login to ykst...");
    let request = tonic::Request::new(OAuthLoginRequest {
        code,
        channel: OAuthLoginChannel::LoginWithJAccount as i32,
        source: LoginSource::Web as i32,
        web_source: WebSource::ProdServer as i32,
    });
    let token = client.o_auth_login(request).await?.into_inner().token;
    println!("Token: {}", token);
    Ok(())
}
