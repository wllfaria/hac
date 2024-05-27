use crate::{
    net::{request_manager::Response, response_decoders::ResponseDecoder},
    text_object::TextObject,
};
use std::{ops::Add, time::Instant};

pub struct JsonDecoder;

#[async_trait::async_trait]
impl ResponseDecoder for JsonDecoder {
    async fn decode(
        &self,
        response: reqwest::Response,
        start: Instant,
    ) -> anyhow::Result<Response> {
        let duration = start.elapsed();
        let headers = response.headers().to_owned();
        let status = response.status();
        let headers_size: u64 = response
            .headers()
            .iter()
            .map(|(k, v)| k.as_str().len().add(v.as_bytes().len()).add(4) as u64)
            .sum();

        let mut body: Option<String> = None;
        let mut pretty_body = None;
        if response.content_length().is_some_and(|len| len.gt(&0)) {
            body = Some(response.json().await?);
            pretty_body = Some(TextObject::from(&serde_json::to_string_pretty(&body)?));
        }

        let body_size = body.as_ref().map(|body| body.len()).unwrap_or_default() as u64;
        let size = headers_size.add(body_size);

        Ok(Response {
            body,
            pretty_body,
            headers,
            duration,
            status,
            size,
            headers_size,
            body_size,
        })
    }
}
