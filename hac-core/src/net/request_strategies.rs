pub mod http_strategy;

use std::future::Future;

use crate::net::request_manager::Response;
use hac_store::collection::Request;

pub trait RequestStrategy {
    fn handle(&self, request: Request) -> impl Future<Output = Response>;
}
