pub mod http_strategy;

use crate::{collection::types::Request, net::request_manager::Response};

#[async_trait::async_trait]
pub trait RequestStrategy {
    async fn handle(&self, request: Request) -> anyhow::Result<Response>;
}
