use crate::{
    net::{request_manager::Response, response_decoders::ResponseDecoder},
    text_object::TextObject,
};
use std::time::Instant;

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
        let body: serde_json::Value = response.json().await?;
        let pretty_body = serde_json::to_string_pretty(&body)?;
        let pretty_body = TextObject::from(&pretty_body);
        let body = body.to_string();

        Ok(Response {
            body,
            pretty_body,
            headers,
            duration,
        })
    }
}
