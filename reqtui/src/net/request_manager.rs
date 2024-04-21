use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use crate::schema::types::Request;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ReqtuiResponse {
    pub body: String,
    pub pretty_body: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ReqtuiNetRequest {
    Request(Request),
    Response(ReqtuiResponse),
}

#[tracing::instrument(skip(response_tx))]
pub fn handle_request(request: Request, response_tx: UnboundedSender<ReqtuiNetRequest>) {
    tracing::debug!("starting to handle user request");
    tokio::spawn(async move {
        match reqwest::get(request.uri).await {
            Ok(res) => {
                tracing::debug!("request handled successfully, sending response");
                let body = res.text().await.expect("failed to decode body");
                let pretty_body = serde_json::to_string_pretty(&body).unwrap();
                response_tx
                    .send(ReqtuiNetRequest::Response(ReqtuiResponse {
                        body,
                        pretty_body,
                    }))
                    .expect("failed to send response through channel");
            }
            Err(_) => todo!(),
        }
    });
}
