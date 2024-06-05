use crate::{
    collection::types::{Request, RequestMethod},
    net::{
        request_manager::Response,
        request_strategies::RequestStrategy,
        response_decoders::{decoder_from_headers, ResponseDecoder},
    },
};

pub struct HttpResponse;

#[async_trait::async_trait]
impl RequestStrategy for HttpResponse {
    async fn handle(&self, request: Request) -> Response {
        let client = reqwest::Client::new();

        match request.method {
            RequestMethod::Get => self.handle_get_request(client, request).await,
            RequestMethod::Post => self.handle_post_request(client, request).await,
            RequestMethod::Put => self.handle_put_request(client, request).await,
            RequestMethod::Patch => self.handle_patch_request(client, request).await,
            RequestMethod::Delete => self.handle_delete_request(client, request).await,
        }
    }
}

impl HttpResponse {
    async fn handle_get_request(&self, client: reqwest::Client, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client.get(request.uri).send().await {
            Ok(response) => {
                let decoder = decoder_from_headers(response.headers());
                decoder.decode(response, now).await
            }
            Err(e) => Response {
                is_error: true,
                cause: Some(e.to_string()),
                body: None,
                pretty_body: None,
                body_size: None,
                size: None,
                headers_size: None,
                status: None,
                headers: None,
                duration: now.elapsed(),
            },
        }
    }

    async fn handle_post_request(&self, client: reqwest::Client, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client
            .post(request.uri)
            .json(&request.body.unwrap_or_default())
            .send()
            .await
        {
            Ok(response) => {
                let decoder = decoder_from_headers(response.headers());
                decoder.decode(response, now).await
            }
            Err(e) => Response {
                is_error: true,
                cause: Some(e.to_string()),
                body: None,
                pretty_body: None,
                body_size: None,
                size: None,
                headers_size: None,
                status: None,
                headers: None,
                duration: now.elapsed(),
            },
        }
    }

    async fn handle_put_request(&self, client: reqwest::Client, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client
            .put(request.uri)
            .json(&request.body.unwrap_or_default())
            .send()
            .await
        {
            Ok(response) => {
                let decoder = decoder_from_headers(response.headers());
                decoder.decode(response, now).await
            }
            Err(e) => Response {
                is_error: true,
                cause: Some(e.to_string()),
                body: None,
                pretty_body: None,
                body_size: None,
                size: None,
                headers_size: None,
                status: None,
                headers: None,
                duration: now.elapsed(),
            },
        }
    }

    async fn handle_patch_request(&self, client: reqwest::Client, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client
            .patch(request.uri)
            .json(&request.body.unwrap_or_default())
            .send()
            .await
        {
            Ok(response) => {
                let decoder = decoder_from_headers(response.headers());
                decoder.decode(response, now).await
            }
            Err(e) => Response {
                is_error: true,
                cause: Some(e.to_string()),
                body: None,
                pretty_body: None,
                body_size: None,
                size: None,
                headers_size: None,
                status: None,
                headers: None,
                duration: now.elapsed(),
            },
        }
    }

    async fn handle_delete_request(&self, client: reqwest::Client, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client
            .delete(request.uri)
            .json(&request.body.unwrap_or_default())
            .send()
            .await
        {
            Ok(response) => {
                let decoder = decoder_from_headers(response.headers());
                decoder.decode(response, now).await
            }
            Err(e) => Response {
                is_error: true,
                cause: Some(e.to_string()),
                body: None,
                pretty_body: None,
                body_size: None,
                size: None,
                headers_size: None,
                status: None,
                headers: None,
                duration: now.elapsed(),
            },
        }
    }
}
