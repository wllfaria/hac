use crate::net::request_client::RequestClient;
use crate::net::request_manager::Response;
use crate::net::request_strategies::RequestStrategy;
use crate::net::response_decoders::{decoder_from_headers, ResponseDecoder};
use hac_store::collection::{ReqMethod, Request};

pub struct HttpResponse;

impl RequestStrategy for HttpResponse {
    async fn handle(&self, request: Request) -> Response {
        let client = RequestClient::default();

        match request.method {
            ReqMethod::Get => self.handle_get_request(client, request).await,
            ReqMethod::Post => self.handle_post_request(client, request).await,
            ReqMethod::Put => self.handle_put_request(client, request).await,
            ReqMethod::Patch => self.handle_patch_request(client, request).await,
            ReqMethod::Delete => self.handle_delete_request(client, request).await,
        }
    }
}

impl HttpResponse {
    async fn handle_get_request(&self, client: RequestClient, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client.get(&request).send().await {
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

    async fn handle_post_request(&self, client: RequestClient, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client.post(&request).json(&request.body).send().await {
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

    async fn handle_put_request(&self, client: RequestClient, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client.put(&request).json(&request.body).send().await {
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

    async fn handle_patch_request(&self, client: RequestClient, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client.patch(&request).json(&request.body).send().await {
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

    async fn handle_delete_request(&self, client: RequestClient, request: Request) -> Response {
        let now = std::time::Instant::now();
        match client.delete(&request).json(&request.body).send().await {
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
