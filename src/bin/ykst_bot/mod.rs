#![allow(dead_code)]

use tonic::{Status, service::{Interceptor, interceptor::InterceptedService}, Request};
use tonic::transport::{Channel, Endpoint};
use model::{*, tree_hole_client::TreeHoleClient};

pub mod model {
    tonic::include_proto!("model");
}

pub struct AuthInterceptor {
    token: String,
}

impl AuthInterceptor {
    pub fn new(token: String) -> Self {
        AuthInterceptor { token }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        // insert treehole token
        request.metadata_mut().insert("authorization", self.token.parse().unwrap());
        Ok(request)
    }
}

#[derive(Debug, Clone)]
pub struct Bot<T> {
    client: TreeHoleClient<T>,
    identity: String,
}

impl Bot<InterceptedService<Channel, AuthInterceptor>> {
    pub async fn new(api_url: String, token: String, identity: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Endpoint::from_shared(api_url)?.connect().await?;
        let client = TreeHoleClient::with_interceptor(channel, AuthInterceptor::new(token));
        let mut bot = Bot { client, identity };
        bot.ping().await?;
        Ok(bot)
    }

    pub async fn ping(&mut self) -> Result<EmptyRequest, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(EmptyRequest {});
        let response = self.client.ping(request).await?.into_inner();
        Ok(response)
    }

    pub async fn get_profile(&mut self) -> Result<User, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(EmptyRequest {});
        let user = self.client.get_profile(request).await?.into_inner();
        Ok(user)
    }

    pub async fn get_user_threads(&mut self) -> Result<ThreadsResponse, Box<dyn std::error::Error>> {
        let request: Request<ThreadsQueryRequest> = tonic::Request::new(Default::default());
        let threads = self.client.get_user_threads(request).await?.into_inner();
        Ok(threads)
    }

    pub async fn create_thread(&mut self, category_id: u64, title: String, content: String) -> Result<Thread, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Thread {
            category_id,
            title,
            content,
            identity_code: self.identity.clone(),
            tags: vec![],
            ..Default::default()
        });
        let thread = self.client.put_thread(request).await?.into_inner();
        Ok(thread)
    }

    pub async fn reply_to_thread(&mut self, thread_id: u64, content: String) -> Result<Post, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Post {
            thread_id,
            content,
            identity_code: self.identity.clone(),
            ..Default::default()
        });
        let post = self.client.put_post(request).await?.into_inner();
        Ok(post)
    }
}