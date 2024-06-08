pub mod http_strategy;

use std::future::Future;

use crate::{collection::types::Request, net::request_manager::Response};

pub trait RequestStrategy {
    fn handle(&self, request: Request) -> impl Future<Output = Response>;
}
