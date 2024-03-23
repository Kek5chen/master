use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use russh::server;
use russh::server::{Auth, Server};

#[derive(Clone)]
struct MowSSH;

impl server::Server for MowSSH {
    type Handler = Self;

    fn new_client(&mut self, _: Option<SocketAddr>) -> Self {
        self.clone()
    }
}

#[async_trait]
impl server::Handler for MowSSH {
    type Error = anyhow::Error;

    async fn auth_password(&mut self, user: &str, password: &str) -> Result<Auth, Self::Error> {
        match user == "cat" && password == "mow" {
            true => Ok(Auth::Accept),
            false => Ok(Auth::Reject { proceed_with_methods: None })
        }
    }
}

#[tokio::main]
async fn main() {
    let config = server::Config::default();
    let config = Arc::new(config);
    let mut sh = MowSSH;
    sh.run_on_address(config, ("0.0.0.0", 2222)).await.unwrap()
}
