mod json_highlight;
mod request;

use crate::request::{Auth, HttpMethod, HttpRequest};
use iced::widget::{
    Scrollable, button, column, horizontal_rule, pick_list, radio, row,
    scrollable::{Direction, Scrollbar, Viewport},
    text, text_editor, text_input,
};

use iced::Task;

fn main() -> iced::Result {
    iced::application("PatchLite", App::update, App::view).run_with(App::new)
}

#[derive(Default)]
struct App {
    url: String,
    method: Option<HttpMethod>,
    request_body: Option<String>,
    request_headers: Vec<(String, String)>,
    response_message: Option<String>,
    response_message_offset: String,
    request: HttpRequest,
    tab: Tab,
    request_body_content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Init,
    UpdateUrl(String),
    SendRequest,
    UpdateMethod(HttpMethod),
    UpdateAuth(Auth),
    Scrolled(Viewport),
    RequestCompleted(Result<String, String>),
    Clear,
    UpdateBody(text_editor::Action),
    UpdateTab(Tab),
    UpdateUsername(String),
    UpdatePassword(String),
    UpdateToken(String),
    UpdateHeaderKey(usize, String),
    UpdateHeaderValue(usize, String),
    RemoveHeaderRow(usize),
    AddHeaderRow,
}

#[derive(Debug, Clone)]
enum Tab {
    None,
    Auth,
    Headers,
    Body,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::None
    }
}
impl Tab {
    pub fn to_int(&self) -> Option<u8> {
        match self {
            Tab::None => Some(0),
            Tab::Auth => Some(1),
            Tab::Headers => Some(2),
            Tab::Body => Some(3),
        }
    }
    pub fn from_int(i: u8) -> Self {
        match i {
            0 => Tab::None,
            1 => Tab::Auth,
            2 => Tab::Headers,
            3 => Tab::Body,
            _ => Tab::None,
        }
    }
}

// #[derive(Default)]
// struct HttpMethodComponent {
//     method: HttpMethod,
//     color: Color,
// }

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Init => {}
            Message::UpdateUrl(new_url) => {
                self.request.url = new_url;
            }
            Message::SendRequest => {
                if self.request.url.is_empty() {
                    println!("URL is empty!");
                }

                self.request.set_headers(&self.request_headers);

                let req = self.request.clone();
                return Task::perform(
                    async move {
                        let result = req.send().await;

                        match result {
                            Ok(response) => {
                                let status = response.status();
                                let body = response.text().await.unwrap_or_default();
                                Ok(format!("Status: {}\nBody:\n{}", status, body))
                            }
                            Err(e) => Err(format!("Request failed: {}", e)),
                        }
                    },
                    Message::RequestCompleted,
                );
            }
            Message::RequestCompleted(result) => match result {
                Ok(response) => {
                    self.response_message = response.into();
                }
                Err(e) => {
                    self.response_message = e.into();
                }
            },
            Message::UpdateMethod(new_method) => {
                self.request.method = Some(new_method);
            }
            Message::UpdateAuth(auth_type) => {
                self.request.auth = auth_type;
            }
            Message::UpdateTab(tab) => {
                self.tab = tab;
            }
            Message::UpdateUsername(username) => {
                self.request.username = username;
            }
            Message::UpdatePassword(password) => {
                self.request.password = password;
            }
            Message::UpdateToken(token) => {
                self.request.token = token;
            }

            Message::UpdateBody(action) => {
                self.request_body_content.perform(action);
                self.request.body = self.request_body_content.text().to_string().into();
            }
            Message::UpdateHeaderKey(i, key) => {
                if let Some(header) = self.request_headers.get_mut(i) {
                    self.request_headers[i].0 = key;
                }
            }
            Message::UpdateHeaderValue(i, value) => {
                if let Some(header) = self.request_headers.get_mut(i) {
                    self.request_headers[i].1 = value;
                }
            }
            Message::RemoveHeaderRow(i) => {
                if i < self.request_headers.len() {
                    self.request_headers.remove(i);
                }
            }
            Message::AddHeaderRow => {
                self.request_headers.push((String::new(), String::new()));
            }
            Message::Scrolled(v) => {
                self.response_message_offset =
                    format!("{} {}", v.absolute_offset().x, v.absolute_offset().y)
            }
            Message::Clear => {
                self.response_message = None;
                self.response_message_offset.clear();
                self.method = None;
                self.url.clear();
                self.request_body = None;
                self.request_headers.clear();
                self.request = HttpRequest::default();
            }
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let method_pick_list = [
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
        ];

        let highlighted_response =
            json_highlight::pretty_json_str(self.response_message.as_deref().unwrap_or(""));

        let response = column([text(highlighted_response).into()]);

        //todo add PaneGrid
        let mut content = column![
            row![
                pick_list(method_pick_list, self.request.method, Message::UpdateMethod,)
                    .placeholder("Select Method"),
                text_input("", self.request.url.as_str()).on_input(|s| Message::UpdateUrl(s)),
                button("Send").on_press(Message::SendRequest),
            ]
            .spacing(10)
            .padding(10),
            horizontal_rule(20),
            row![
                radio("Closed", 0, self.tab.to_int(), |i| {
                    Message::UpdateTab(Tab::from_int(i))
                }),
                radio("Auth", 1, self.tab.to_int(), |i| {
                    Message::UpdateTab(Tab::from_int(i))
                }),
                radio("Headers", 2, self.tab.to_int(), |i| {
                    Message::UpdateTab(Tab::from_int(i))
                }),
                radio("Body", 3, self.tab.to_int(), |i| {
                    Message::UpdateTab(Tab::from_int(i))
                })
            ]
            .spacing(10)
            .padding(10),
            horizontal_rule(50),
        ];

        match self.tab {
            Tab::None => {}
            Tab::Auth => {
                content = content.push(column![
                    row![
                        radio("No Auth", 0, self.request.auth.to_int(), |i| {
                            Message::UpdateAuth(Auth::from_int(i))
                        }),
                        radio("Basic", 1, self.request.auth.to_int(), |i| {
                            Message::UpdateAuth(Auth::from_int(i))
                        }),
                        radio("Bearer", 2, self.request.auth.to_int(), |i| {
                            Message::UpdateAuth(Auth::from_int(i))
                        }),
                    ]
                    .spacing(10)
                    .padding(10),
                    horizontal_rule(50),
                ]);
                match self.request.auth {
                    Auth::Basic => {
                        content = content.push(
                            column![
                                text("Basic Authentication selected."),
                                text_input("Username", self.request.username.as_str())
                                    .on_input(|s| Message::UpdateUsername(s)),
                                text_input("Password", self.request.password.as_str())
                                    .on_input(|s| Message::UpdatePassword(s)),
                            ]
                            .spacing(10)
                            .padding(10),
                        );
                    }
                    Auth::Bearer => {
                        content = content.push(
                            column![
                                text("Bearer Authentication selected."),
                                text_input("Token", self.request.token.as_str())
                                    .on_input(|s| Message::UpdateToken(s)),
                            ]
                            .spacing(10)
                            .padding(10),
                        );
                    }
                    Auth::None => {}
                }
            }
            Tab::Headers => {
                content = content.push(
                    column![
                        text("Headers configuration will go here."),
                        row![
                            text("Key"),
                            text("Value"),
                            text("       "),
                            button("Add Header +").on_press(Message::AddHeaderRow),
                        ]
                        .spacing(10)
                        .padding(10),
                    ]
                    .spacing(10)
                    .padding(10),
                );
                for (i, (key, value)) in self.request_headers.iter().enumerate() {
                    content = content.push(
                        row![
                            text_input("", key.as_str())
                                .on_input(move |k| Message::UpdateHeaderKey(i, k)),
                            text_input("", value.as_str())
                                .on_input(move |v| Message::UpdateHeaderValue(i, v)),
                            button("-").on_press(Message::RemoveHeaderRow(i)),
                        ]
                        .spacing(10),
                    );
                }
            }
            Tab::Body => {
                content = content.push(
                    column![
                        text("Request Body:"),
                        text_editor(&self.request_body_content)
                            .placeholder("Type something here...")
                            .on_action(Message::UpdateBody),
                    ]
                    .spacing(10)
                    .padding(10),
                );
            }
        }

        content = content.push(horizontal_rule(50));

        content = content.push(
            column![
                Scrollable::new(response)
                    .width(1000)
                    .height(1000)
                    .direction(Direction::Vertical(Scrollbar::new()))
                    .on_scroll(Message::Scrolled),
                text(&self.response_message_offset),
            ]
            .spacing(20),
        );

        //content = content.push(row![button("Clear").on_press(Message::Clear),]);

        content.into()
    }

    fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.request.set_default_headers();
        app.request_headers = app
            .request
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect();
        let task = Task::perform(async {}, |_| Message::Init);
        (app, task)
    }
}

// fn theme(state: &App) -> Theme {
//     Theme::TokyoNight
// }
