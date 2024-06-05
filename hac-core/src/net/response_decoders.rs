mod json_decoder;

use crate::net::request_manager::{ContentType, Response};
use crate::net::response_decoders::json_decoder::JsonDecoder;

use std::time::Instant;

use reqwest::header::HeaderMap;

#[async_trait::async_trait]
pub trait ResponseDecoder {
    async fn decode(&self, response: reqwest::Response, start: Instant) -> Response;
}

pub fn decoder_from_headers(headers: &HeaderMap) -> impl ResponseDecoder {
    match headers.get("Content-Type") {
        Some(header) => match ContentType::from(header.to_str().unwrap_or_default()) {
            ContentType::ApplicationJson => JsonDecoder,
            _ => JsonDecoder,
        },
        None => JsonDecoder,
    }
}
