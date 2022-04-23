use model::tree_hole_client::TreeHoleClient;
use model::*;

pub mod model {
    tonic::include_proto!("model");
}

const API_URL: &str = "https://api.treehole.dyweb.sjtu.cn";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TreeHoleClient::connect(API_URL).await?;

    println!("ping server...");
    let request = tonic::Request::new(EmptyRequest {});
    let response = client.ping(request).await?;
    println!("RESPONSE={:?}", response);

    println!("fetching oauth config...");
    let request = tonic::Request::new(OAuthConfigRequest {
        channel: OAuthLoginChannel::LoginWithJAccount as i32,
        source: LoginSource::Web as i32,
    });
    let response = client.get_o_auth_config(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}
