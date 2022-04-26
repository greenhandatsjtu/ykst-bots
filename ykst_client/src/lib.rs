#![allow(dead_code)]

use tonic::{Status, service::{Interceptor, interceptor::InterceptedService}, Request};
use tonic::transport::{Channel, Endpoint};
use model::{*, tree_hole_client::TreeHoleClient};

pub mod model {
    tonic::include_proto!("model");
}

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
pub enum Category {
    // 综合版
    Main = 1,
    // 校园
    School,
    // 深夜食堂
    Canteen,
    // 情感
    Emotion,
    // 学业
    Study,
    // 科技
    Tech,
    // 值班室
    DutyRoom,
    // 游戏
    Game,
    // 深水区
    Deep,
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
pub struct Client<T> {
    client: TreeHoleClient<T>,
    pub identity: String,
}

impl Client<InterceptedService<Channel, AuthInterceptor>> {
    pub async fn new(api_url: String, token: String, identity: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Endpoint::from_shared(api_url)?.connect().await?;
        let client = TreeHoleClient::with_interceptor(channel, AuthInterceptor::new(token));
        let mut c = Client { client, identity };
        c.ping().await?;
        Ok(c)
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

    pub async fn create_thread(&mut self, category: Category, title: String, content: String, tags: Option<Vec<Tag>>) -> Result<Thread, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Thread {
            category_id: category as u64,
            title,
            content,
            identity_code: self.identity.clone(),
            tags: tags.unwrap_or(vec![]),
            ..Default::default()
        });
        let thread = self.client.put_thread(request).await?.into_inner();
        Ok(thread)
    }

    pub async fn reply_to_post(&mut self, thread_id: u64, post_id: Option<u64>, content: String) -> Result<Post, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Post {
            thread_id,
            reply_to_post_id: post_id,
            content,
            identity_code: self.identity.clone(),
            ..Default::default()
        });
        let post = self.client.put_post(request).await?.into_inner();
        Ok(post)
    }

    pub async fn reply_to_thread(&mut self, thread_id: u64, content: String) -> Result<Post, Box<dyn std::error::Error>> {
        self.reply_to_post(thread_id, None, content).await
    }

    pub async fn get_thread_replies(&mut self, thread_id: u64, last: u64, size: u32) -> Result<PostsResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PostsQueryRequest {
            thread_id,
            last,
            size,
            ..Default::default()
        });
        let posts = self.client.get_thread_posts(request).await?.into_inner();
        Ok(posts)
    }

    pub async fn appreciate_thread(&mut self, thread_id: u64, amount: i32) -> Result<Thread, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(AppreciateRequest {
            id: thread_id,
            amount,
        });
        let thread = self.client.appreciate_thread(request).await?.into_inner();
        Ok(thread)
    }

    pub async fn appreciate_post(&mut self, post_id: u64, amount: i32) -> Result<Post, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(AppreciateRequest {
            id: post_id,
            amount,
        });
        let post = self.client.appreciate_post(request).await?.into_inner();
        Ok(post)
    }
}