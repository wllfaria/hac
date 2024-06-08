mod json_decoder;

use crate::net::request_manager::{ContentType, Response};
use crate::net::response_decoders::json_decoder::JsonDecoder;

use std::future::Future;
use std::time::Instant;

use reqwest::header::HeaderMap;

pub trait ResponseDecoder {
    fn decode(
        &self,
        response: reqwest::Response,
        start: Instant,
    ) -> impl Future<Output = Response> + Send;
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
