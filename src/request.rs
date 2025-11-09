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

// Insert headers example:
// data.headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
// data.headers.insert(USER_AGENT, HeaderValue::from_static("PatchLite/0.1"));

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

    pub async fn send(&self) -> Result<Response, Error> {
        let api_client = reqwest::Client::new();
        match self.method {
            Some(m) => match m {
                HttpMethod::GET => {
                    let mut req = api_client
                        .get(self.url.clone())
                        .headers(self.headers.clone());

                    req = match self.auth {
                        Auth::None => req,
                        Auth::Bearer => req.bearer_auth(self.token.clone()),
                        Auth::Basic => {
                            req.basic_auth(self.username.clone(), Some(self.password.clone()))
                        }
                    };

                    req.send().await
                }
                HttpMethod::POST => {
                    let mut req = api_client
                        .post(self.url.clone())
                        .headers(self.headers.clone());

                    req = match self.auth {
                        Auth::None => req,
                        Auth::Bearer => req.bearer_auth(self.token.clone()),
                        Auth::Basic => {
                            req.basic_auth(self.username.clone(), Some(self.password.clone()))
                        }
                    };

                    if let Some(body) = self.body.as_ref().filter(|b| !b.trim().is_empty()) {
                        if serde_json::from_str::<serde_json::Value>(body).is_ok() {
                            req = req.body(body.clone());
                        }
                    }

                    req.send().await
                }
                HttpMethod::PUT => {
                    let mut req = api_client
                        .put(self.url.clone())
                        .headers(self.headers.clone());

                    req = match self.auth {
                        Auth::None => req,
                        Auth::Bearer => req.bearer_auth(self.token.clone()),
                        Auth::Basic => {
                            req.basic_auth(self.username.clone(), Some(self.password.clone()))
                        }
                    };

                    if self.body.is_some() && !self.body.as_ref().unwrap().is_empty() {
                        req = req.body(self.body.as_ref().unwrap().clone());
                    }

                    req.send().await
                }
                HttpMethod::PATCH => {
                    let mut req = api_client
                        .patch(self.url.clone())
                        .headers(self.headers.clone());

                    req = match self.auth {
                        Auth::None => req,
                        Auth::Bearer => req.bearer_auth(self.token.clone()),
                        Auth::Basic => req.basic_auth("admin", Some("good password")),
                    };

                    if self.body.is_some() && !self.body.as_ref().unwrap().is_empty() {
                        req = req.body(self.body.as_ref().unwrap().clone());
                    }

                    req.send().await
                }
                HttpMethod::DELETE => {
                    let mut req = api_client
                        .delete(self.url.clone())
                        .headers(self.headers.clone());

                    req = match self.auth {
                        Auth::None => req,
                        Auth::Bearer => req.bearer_auth(self.token.clone()),
                        Auth::Basic => {
                            req.basic_auth(self.username.clone(), Some(self.password.clone()))
                        }
                    };

                    if self.body.is_some() && !self.body.as_ref().unwrap().is_empty() {
                        req = req.body(self.body.as_ref().unwrap().clone());
                    }

                    req.send().await
                }
            },
            None => {
                let result = reqwest::get("http://url_invalida###").await;
                result
            }
        }
    }
}
