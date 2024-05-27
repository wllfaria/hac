use crate::{
    collection::types::{BodyType, Request},
    net::request_strategies::{http_strategy::HttpResponse, RequestStrategy},
    text_object::{Readonly, TextObject},
};
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, PartialEq)]
pub struct Response {
    pub body: Option<String>,
    pub pretty_body: Option<TextObject<Readonly>>,
    pub headers: HeaderMap<HeaderValue>,
    pub duration: Duration,
    pub status: reqwest::StatusCode,
    pub headers_size: u64,
    pub body_size: u64,
    pub size: u64,
}

pub struct RequestManager;

impl RequestManager {
    pub async fn handle<S>(strategy: S, request: Request) -> anyhow::Result<Response>
    where
        S: RequestStrategy,
    {
        let response = strategy.handle(request).await?;
        Ok(response)
    }
}

pub enum ContentType {
    TextPlain,
    TextHtml,
    TextCss,
    TextJavascript,
    ApplicationJson,
    ApplicationXml,
}

impl From<&str> for ContentType {
    fn from(value: &str) -> Self {
        match value {
            _ if value.to_ascii_lowercase().contains("application/json") => Self::ApplicationJson,
            _ if value.to_ascii_lowercase().contains("application/xml") => Self::ApplicationXml,
            _ if value.to_ascii_lowercase().contains("text/plain") => Self::TextPlain,
            _ if value.to_ascii_lowercase().contains("text/plain") => Self::TextPlain,
            _ if value.to_ascii_lowercase().contains("text/html") => Self::TextHtml,
            _ if value.to_ascii_lowercase().contains("text/css") => Self::TextCss,
            _ if value.to_ascii_lowercase().contains("text/javascript") => Self::TextJavascript,
            _ => Self::TextPlain,
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn handle_request(request: Request, response_tx: UnboundedSender<Response>) {
    tracing::debug!("starting to handle user request");
    tokio::spawn(async move {
        let result = match request.body_type.as_ref() {
            // if we dont have a body type, this is a GET request, so we use HTTP strategy
            None => RequestManager::handle(HttpResponse, request).await,
            Some(body_type) => match body_type {
                BodyType::Json => RequestManager::handle(HttpResponse, request).await,
            },
        };

        match result {
            Ok(response) => response_tx.send(response).ok(),
            Err(e) => {
                tracing::error!("{e:?}");
                todo!();
            }
        }
    });
}
