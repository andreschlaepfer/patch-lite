use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Error, Response};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Auth {
    None,
    Basic,
    Bearer,
}

impl Auth {
    pub fn to_int(&self) -> Option<u8> {
        match self {
            Auth::None => Some(0),
            Auth::Basic => Some(1),
            Auth::Bearer => Some(2),
        }
    }
    pub fn from_int(i: u8) -> Self {
        match i {
            0 => Auth::None,
            1 => Auth::Basic,
            2 => Auth::Bearer,
            _ => Auth::None,
        }
    }
}

impl Default for Auth {
    fn default() -> Self {
        Auth::None
    }
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::GET
    }
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
        }
        .to_string()
    }
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "PATCH" => Some(HttpMethod::PATCH),
            "DELETE" => Some(HttpMethod::DELETE),
            _ => None,
        }
    }
}

#[derive(Default, Clone)]
pub struct HttpRequest {
    pub method: Option<HttpMethod>,
    pub url: String,
    pub body: Option<String>,
    pub auth: Auth,
    pub token: String,
    pub username: String,
    pub password: String,
    pub headers: HeaderMap,
}

impl HttpRequest {
    pub fn set_default_headers(&mut self) {
        self.headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    }

    pub fn set_headers(&mut self, headers_vec: &Vec<(String, String)>) {
        let mut header_map = HeaderMap::new();
        for (key, value) in headers_vec {
            if let Ok(header_name) = key.parse::<HeaderName>() {
                if let Ok(header_value) = value.parse() {
                    header_map.insert(header_name, header_value);
                }
            }
        }
        self.headers = header_map;
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.auth {
            Auth::None => req,
            Auth::Bearer => req.bearer_auth(self.token.clone()),
            Auth::Basic => req.basic_auth(self.username.clone(), Some(self.password.clone())),
        }
    }

    fn apply_body(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.body.as_ref().filter(|b| !b.trim().is_empty()) {
            Some(body) => req.body(body.clone()),
            None => req,
        }
    }

    pub async fn send(&self) -> Result<Response, Error> {
        let client = reqwest::Client::new();
        let method = self.method.unwrap_or(HttpMethod::GET);

        let builder = match method {
            HttpMethod::GET => client.get(&self.url),
            HttpMethod::POST => client.post(&self.url),
            HttpMethod::PUT => client.put(&self.url),
            HttpMethod::PATCH => client.patch(&self.url),
            HttpMethod::DELETE => client.delete(&self.url),
        };

        let builder = builder.headers(self.headers.clone());
        let builder = self.apply_auth(builder);
        let builder = match method {
            HttpMethod::GET => builder,
            _ => self.apply_body(builder),
        };

        builder.send().await
    }
}
