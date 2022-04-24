use model::tree_hole_client::TreeHoleClient;
use model::*;
use tonic::{Status, transport::{Endpoint}};
use config::Config;
use tonic::service::{Interceptor};

pub mod model {
    tonic::include_proto!("model");
}

struct AuthInterceptor {
    token: String
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        // insert treehole token
        request.metadata_mut().insert("authorization", self.token.parse().unwrap());
        Ok(request)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name("config.yaml"))
        .build()?;
    let api_url = settings.get_string("API_URL")?;
    let token = settings.get_string("TREEHOLE_TOKEN")?;

    let channel = Endpoint::from_shared(api_url)?.connect().await?;
    let mut client = TreeHoleClient::with_interceptor(channel, AuthInterceptor { token });

    println!("Ping server...");
    let request = tonic::Request::new(EmptyRequest {});
    let response = client.ping(request).await?;
    println!("RESPONSE={:?}", response);

    println!("Get profile...");
    let request = tonic::Request::new(EmptyRequest {});
    let response = client.get_profile(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}
