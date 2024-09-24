use crate::net::request_strategies::{http_strategy::HttpResponse, RequestStrategy};
use crate::text_object::{Readonly, TextObject};
use hac_store::collection::{BodyKind, Request};

use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, PartialEq)]
pub struct Response {
    pub body: Option<String>,
    pub pretty_body: Option<TextObject<Readonly>>,
    pub headers: Option<HeaderMap<HeaderValue>>,
    pub duration: Duration,
    pub status: Option<reqwest::StatusCode>,
    pub headers_size: Option<u64>,
    pub body_size: Option<u64>,
    pub size: Option<u64>,
    pub is_error: bool,
    pub cause: Option<String>,
}

pub struct RequestManager;

impl RequestManager {
    pub async fn handle<S>(strategy: S, request: Request) -> Response
    where
        S: RequestStrategy,
    {
        strategy.handle(request).await
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
    tokio::spawn(async move {
        let response = match request.body_kind {
            // if we dont have a body type, this is a GET request, so we use HTTP strategy
            BodyKind::NoBody => RequestManager::handle(HttpResponse, request).await,
            BodyKind::Json => RequestManager::handle(HttpResponse, request).await,
        };

        response_tx.send(response).is_err().then(|| std::process::abort());
    });
}
