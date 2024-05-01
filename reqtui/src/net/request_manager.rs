use std::collections::HashMap;

use ratatui::style::Style;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    schema::types::Request,
    syntax::highlighter::HIGHLIGHTER,
    text_object::{Readonly, TextObject},
};

#[derive(Debug, PartialEq)]
pub struct ReqtuiResponse {
    pub body: String,
    pub pretty_body: TextObject<Readonly>,
}

#[derive(Debug, PartialEq)]
pub enum ReqtuiNetRequest {
    Request(Request),
    Response(ReqtuiResponse),
    Error(String),
}

#[tracing::instrument(skip(response_tx))]
pub fn handle_request(
    request: Request,
    response_tx: UnboundedSender<ReqtuiNetRequest>,
    tokens: HashMap<String, Style>,
) {
    tracing::debug!("starting to handle user request");
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        match client.get(request.uri).send().await {
            Ok(res) => {
                tracing::debug!("request handled successfully, sending response");
                let body: serde_json::Value = res
                    .json()
                    .await
                    .map_err(|_| {
                        response_tx.send(ReqtuiNetRequest::Error(
                            "failed to decode json response".into(),
                        ))
                    })
                    .expect("failed to send response through channel");

                let pretty_body = serde_json::to_string_pretty(&body)
                    .map_err(|_| {
                        response_tx.send(ReqtuiNetRequest::Error(
                            "failed to decode json response".into(),
                        ))
                    })
                    .expect("failed to send response through channel");

                let mut highlighter = HIGHLIGHTER.write().unwrap();
                let body = body.to_string();

                let tree = highlighter.parse(&pretty_body);
                let highlight = highlighter.apply(&pretty_body, tree.as_ref(), &tokens);
                let pretty_body = TextObject::from(&pretty_body).with_highlight(highlight);

                response_tx
                    .send(ReqtuiNetRequest::Response(ReqtuiResponse {
                        body: body.to_string(),
                        pretty_body,
                    }))
                    .expect("failed to send response through channel");
            }
            Err(_) => todo!(),
        }
    });
}
