use hac_store::collection::Request;

#[derive(Debug)]
pub struct RequestClient {
    client: reqwest::Client,
}

impl RequestClient {
    pub fn new() -> Self {
        RequestClient {
            client: reqwest::Client::new(),
        }
    }

    pub fn get(&self, request: &Request) -> reqwest::RequestBuilder {
        let request_builder = self.client.get(&request.uri);
        self.append_headers(request, request_builder)
    }

    pub fn post(&self, request: &Request) -> reqwest::RequestBuilder {
        let request_builder = self.client.post(&request.uri);
        self.append_headers(request, request_builder)
    }

    pub fn put(&self, request: &Request) -> reqwest::RequestBuilder {
        let request_builder = self.client.put(&request.uri);
        self.append_headers(request, request_builder)
    }

    pub fn patch(&self, request: &Request) -> reqwest::RequestBuilder {
        let request_builder = self.client.patch(&request.uri);
        self.append_headers(request, request_builder)
    }

    pub fn delete(&self, request: &Request) -> reqwest::RequestBuilder {
        let request_builder = self.client.delete(&request.uri);
        self.append_headers(request, request_builder)
    }

    fn append_headers(
        &self,
        request: &Request,
        mut request_builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        for header in request.headers.iter().filter(|header| header.enabled) {
            let header_name = header.key.clone();
            let header_value = header.val.clone();
            request_builder = request_builder.header(header_name, header_value);
        }

        request_builder
    }
}

impl Default for RequestClient {
    fn default() -> Self {
        Self::new()
    }
}
