use iced::alignment::{Horizontal, Vertical};
use iced::theme::{self};
use iced::widget::{
    Column, Row, button, checkbox, column, container, pick_list, row, scrollable, text,
    text_editor, text_input,
};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};

use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> iced::Result {
    PatchLiteApp::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

/* =============================
Domain / Models
============================= */

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}
static METHODS: &[HttpMethod] = &[
    HttpMethod::GET,
    HttpMethod::POST,
    HttpMethod::PUT,
    HttpMethod::DELETE,
    HttpMethod::PATCH,
    HttpMethod::HEAD,
    HttpMethod::OPTIONS,
];
impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HttpMethod::*;
        write!(
            f,
            "{}",
            match self {
                GET => "GET",
                POST => "POST",
                PUT => "PUT",
                DELETE => "DELETE",
                PATCH => "PATCH",
                HEAD => "HEAD",
                OPTIONS => "OPTIONS",
            }
        )
    }
}
impl From<HttpMethod> for Method {
    fn from(m: HttpMethod) -> Self {
        use HttpMethod::*;
        match m {
            GET => Method::GET,
            POST => Method::POST,
            PUT => Method::PUT,
            DELETE => Method::DELETE,
            PATCH => Method::PATCH,
            HEAD => Method::HEAD,
            OPTIONS => Method::OPTIONS,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SavedRequest {
    name: String,
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    body: String,
}

impl Default for SavedRequest {
    fn default() -> Self {
        Self {
            name: "New Request".into(),
            method: "GET".into(),
            url: "https://httpbin.org/get".into(),
            headers: BTreeMap::new(),
            body: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct ResponseView {
    status_line: String,
    duration_ms: u128,
    headers: Vec<(String, String)>,
    body_pretty: String,
}

/* =============================
App State
============================= */

static HTTP: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .use_rustls_tls()
        .build()
        .expect("reqwest client")
});

struct PatchLiteApp {
    // Sidebar
    requests: Vec<SavedRequest>,
    selected: usize,

    // Editor state
    method: HttpMethod,
    url: String,
    headers_kv: Vec<(String, String, bool)>, // (key, value, enabled)
    body_editor: text_editor::Content,
    pretty_json: bool,

    // Response
    response: Option<ResponseView>,

    // Busy
    in_flight: bool,

    // Status/error
    last_error: Option<String>,
}

#[derive(Clone, Debug)]
enum Msg {
    SidebarSelect(usize),
    MethodChanged(HttpMethod),
    UrlChanged(String),
    ToggleHeader(usize, bool),
    HeaderKeyChanged(usize, String),
    HeaderValChanged(usize, String),
    AddHeader,
    RemoveHeader(usize),
    BodyChanged(text_editor::Action),
    TogglePrettyJson(bool),

    SendClicked,
    RequestDone(Result<ResponseView, String>),
}

impl Application for PatchLiteApp {
    type Executor = iced::executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        // Carrega/seed inicial (aqui fixo; depois substitua por persistência em arquivo/SQLite)
        let mut requests = vec![
            SavedRequest {
                name: "GET httpbin".into(),
                ..Default::default()
            },
            SavedRequest {
                name: "POST echo".into(),
                method: "POST".into(),
                url: "https://httpbin.org/post".into(),
                ..Default::default()
            },
        ];
        let selected = 0;
        let chosen = requests.get(selected).cloned().unwrap_or_default();

        let headers_kv = if chosen.headers.is_empty() {
            vec![("User-Agent".into(), "PatchLite/0.1".into(), true)]
        } else {
            chosen
                .headers
                .into_iter()
                .map(|(k, v)| (k, v, true))
                .collect()
        };

        let method = match chosen.method.as_str() {
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        };

        (
            Self {
                requests,
                selected,
                method,
                url: chosen.url,
                headers_kv,
                body_editor: text_editor::Content::with_text(&chosen.body),
                pretty_json: true,
                response: None,
                in_flight: false,
                last_error: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "PatchLite (Iced)".into()
    }

    fn theme(&self) -> Theme {
        theme::CatppuccinMocha
    }

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::SidebarSelect(i) => {
                if let Some(r) = self.requests.get(i).cloned() {
                    self.selected = i;
                    self.method = match r.method.as_str() {
                        "POST" => HttpMethod::POST,
                        "PUT" => HttpMethod::PUT,
                        "DELETE" => HttpMethod::DELETE,
                        "PATCH" => HttpMethod::PATCH,
                        "HEAD" => HttpMethod::HEAD,
                        "OPTIONS" => HttpMethod::OPTIONS,
                        _ => HttpMethod::GET,
                    };
                    self.url = r.url;
                    self.headers_kv = if r.headers.is_empty() {
                        vec![("User-Agent".into(), "PatchLite/0.1".into(), true)]
                    } else {
                        r.headers.into_iter().map(|(k, v)| (k, v, true)).collect()
                    };
                    self.body_editor = text_editor::Content::with_text(&r.body);
                    self.response = None;
                }
            }
            Msg::MethodChanged(m) => self.method = m,
            Msg::UrlChanged(u) => self.url = u,
            Msg::ToggleHeader(idx, en) => {
                if let Some(h) = self.headers_kv.get_mut(idx) {
                    h.2 = en;
                }
            }
            Msg::HeaderKeyChanged(idx, k) => {
                if let Some(h) = self.headers_kv.get_mut(idx) {
                    h.0 = k;
                }
            }
            Msg::HeaderValChanged(idx, v) => {
                if let Some(h) = self.headers_kv.get_mut(idx) {
                    h.1 = v;
                }
            }
            Msg::AddHeader => self.headers_kv.push(("".into(), "".into(), true)),
            Msg::RemoveHeader(idx) => {
                if idx < self.headers_kv.len() {
                    self.headers_kv.remove(idx);
                }
            }
            Msg::BodyChanged(action) => {
                self.body_editor.perform(action);
            }
            Msg::TogglePrettyJson(v) => self.pretty_json = v,

            Msg::SendClicked => {
                if self.in_flight {
                    return Command::none();
                }
                self.in_flight = true;
                self.response = None;
                self.last_error = None;

                let method = self.method;
                let url = self.url.clone();
                let headers: Vec<(String, String)> = self
                    .headers_kv
                    .iter()
                    .filter(|(_, _, enabled)| *enabled)
                    .filter(|(k, _, _)| !k.trim().is_empty())
                    .map(|(k, v, _)| (k.clone(), v.clone()))
                    .collect();
                let body = self.body_editor.text().to_string();

                return Command::perform(
                    async move { send_request(method, url, headers, body).await },
                    |r| match r {
                        Ok(rv) => Msg::RequestDone(Ok(rv)),
                        Err(e) => Msg::RequestDone(Err(e)),
                    },
                );
            }

            Msg::RequestDone(res) => {
                self.in_flight = false;
                match res {
                    Ok(rv) => self.response = Some(rv),
                    Err(e) => self.last_error = Some(e),
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        // Sidebar (lista simples de requests)
        let mut side_col = column![
            text("Collections")
                .size(16)
                .horizontal_alignment(Horizontal::Center)
                .width(Length::Fill),
        ]
        .spacing(8)
        .padding(8)
        .width(200);

        for (i, r) in self.requests.iter().enumerate() {
            let is_sel = i == self.selected;
            let btn = button(
                text(&r.name)
                    .horizontal_alignment(Horizontal::Left)
                    .width(Length::Fill),
            )
            .style(if is_sel {
                theme::Button::Primary
            } else {
                theme::Button::Secondary
            })
            .on_press(Msg::SidebarSelect(i))
            .width(Length::Fill);
            side_col = side_col.push(btn);
        }

        let sidebar = container(scrollable(side_col))
            .width(Length::Fixed(220.0))
            .height(Length::Fill);

        // Editor (método, URL, headers, body)
        let method_pick = pick_list(METHODS, Some(self.method), Msg::MethodChanged).width(120);
        let url_input = text_input("https://example.com", &self.url)
            .on_input(Msg::UrlChanged)
            .padding(8)
            .width(Length::Fill);

        let send_btn = button(if self.in_flight { "Sending..." } else { "Send" })
            .on_press_maybe((!self.in_flight).then_some(Msg::SendClicked))
            .padding([8, 16]);

        let top_bar = row![method_pick, url_input, send_btn]
            .spacing(8)
            .align_items(Alignment::Center);

        let mut headers_col: Column<Msg> =
            column![row![text("Headers"), button("+").on_press(Msg::AddHeader)].spacing(8)]
                .spacing(6);

        for (idx, (k, v, enabled)) in self.headers_kv.iter().cloned().enumerate() {
            headers_col = headers_col.push(
                row![
                    checkbox("", enabled, move |on| Msg::ToggleHeader(idx, on)),
                    text_input("Key", &k)
                        .on_input(move |s| Msg::HeaderKeyChanged(idx, s))
                        .width(160),
                    text_input("Value", &v)
                        .on_input(move |s| Msg::HeaderValChanged(idx, s))
                        .width(Length::Fill),
                    button("x").on_press(Msg::RemoveHeader(idx)),
                ]
                .spacing(6)
                .align_items(Alignment::Center),
            );
        }

        let body_label = row![
            text("Body"),
            checkbox("Pretty JSON", self.pretty_json, Msg::TogglePrettyJson)
        ]
        .spacing(12)
        .align_items(Alignment::Center);

        let body_editor = container(
            text_editor(&self.body_editor)
                .on_action(Msg::BodyChanged)
                .height(Length::Fixed(180.0)),
        )
        .padding(6)
        .style(theme::Container::Box);

        let editor = column![top_bar, headers_col, body_label, body_editor]
            .spacing(12)
            .padding(12)
            .width(Length::Fill);

        // Resposta
        let response_view: Element<_> = if let Some(rv) = &self.response {
            let headers_str = rv
                .headers
                .iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join("\n");

            let meta = text(format!("{}  •  {} ms", rv.status_line, rv.duration_ms)).size(16);

            let headers = container(
                scrollable(text(headers_str).size(14).line_height(1.3))
                    .height(Length::Fixed(120.0)),
            )
            .padding(6)
            .style(theme::Container::Box);

            let body = container(
                scrollable(text(&rv.body_pretty).size(14).line_height(1.3)).height(Length::Fill),
            )
            .padding(6)
            .style(theme::Container::Box);

            column![
                meta,
                text("Headers").size(14),
                headers,
                text("Body").size(14),
                body
            ]
            .spacing(8)
            .into()
        } else if let Some(err) = &self.last_error {
            text(err)
                .style(theme::Text::Color([0.95, 0.3, 0.3].into()))
                .into()
        } else {
            text("No response yet.").into()
        };

        let right = column![editor, response_view]
            .spacing(12)
            .width(Length::Fill)
            .height(Length::Fill);

        row![sidebar, container(right).width(Length::Fill)]
            .height(Length::Fill)
            .into()
    }
}

/* =============================
Networking
============================= */

async fn send_request(
    method: HttpMethod,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
) -> Result<ResponseView, String> {
    let method_req: Method = method.into();

    let mut builder = HTTP.request(method_req, &url);

    // Headers
    for (k, v) in headers {
        if let Ok(name) = reqwest::header::HeaderName::try_from(k.trim().to_string()) {
            if let Ok(val) = reqwest::header::HeaderValue::from_str(v.trim()) {
                builder = builder.header(name, val);
            }
        }
    }

    // Body (só envia em métodos que comportam)
    match method {
        HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH => {
            builder = builder.body(body);
        }
        _ => {}
    }

    let start = Instant::now();
    let resp = builder
        .send()
        .await
        .map_err(|e| format!("Request error: {e}"))?;
    let duration = start.elapsed().as_millis();

    let status_line = format!("{:?} {}", resp.version(), resp.status());

    // Headers
    let mut hdrs = Vec::new();
    for (k, v) in resp.headers().iter() {
        hdrs.push((k.to_string(), v.to_str().unwrap_or("<binary>").to_string()));
    }

    // Body (try pretty JSON)
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Read body error: {e}"))?;

    let body_pretty = if let Ok(text) = std::str::from_utf8(&bytes) {
        // Se marcar pretty e for JSON, formata
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| text.to_string())
        } else {
            text.to_string()
        }
    } else {
        format!("<{} bytes of binary data>", bytes.len())
    };

    Ok(ResponseView {
        status_line,
        duration_ms: duration,
        headers: hdrs,
        body_pretty,
    })
}
