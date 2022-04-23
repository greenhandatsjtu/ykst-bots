use model::tree_hole_client::TreeHoleClient;
use model::{EmptyRequest};

pub mod model {
    tonic::include_proto!("model");
}

const API_URL: &str = "https://api.treehole.dyweb.sjtu.cn";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TreeHoleClient::connect(API_URL).await?;
    let request = tonic::Request::new(EmptyRequest {});
    let response = client.ping(request).await?;
    println!("RESPONSE={:?}", response);
    Ok(())
}
