use crate::net::{request_manager::Response, response_decoders::ResponseDecoder};
use crate::text_object::TextObject;

use std::{ops::Add, time::Instant};

pub struct JsonDecoder;

#[async_trait::async_trait]
impl ResponseDecoder for JsonDecoder {
    async fn decode(&self, response: reqwest::Response, start: Instant) -> Response {
        let duration = start.elapsed();
        let headers = Some(response.headers().to_owned());
        let status = Some(response.status());
        let headers_size: u64 = response
            .headers()
            .iter()
            .map(|(k, v)| k.as_str().len().add(v.as_bytes().len()).add(4) as u64)
            .sum();

        let mut body: Option<String> = None;
        let mut pretty_body = None;

        if response.content_length().is_some_and(|len| len.gt(&0)) {
            if let Ok(body_str) = response.text().await {
                let pretty_body_str = jsonxf::pretty_print(&body_str).unwrap_or_default();
                pretty_body = Some(TextObject::from(&pretty_body_str));
                body = Some(body_str);
            };
        }

        let body_size = body.as_ref().map(|body| body.len()).unwrap_or_default() as u64;
        let size = headers_size.add(body_size);

        Response {
            body,
            pretty_body,
            headers,
            duration,
            status,
            size: Some(size),
            headers_size: Some(headers_size),
            body_size: Some(body_size),
            cause: None,
            is_error: false,
        }
    }
}
