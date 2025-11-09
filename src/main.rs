use std::vec;
mod json_highlight;
mod request;

use crate::request::{Auth, HttpMethod, HttpRequest};
use iced::application::View;
use iced::{
    Background, Color, Font,
    font::Family,
    widget::{
        PickList, Rule, Scrollable, Space, TextInput, button, column, horizontal_rule, pick_list,
        radio, row,
        scrollable::{Direction, Scrollbar, Viewport},
        text, text_input,
        text_input::{Icon, Side},
        vertical_space,
    },
};

use iced::{Task, Theme};
use reqwest::{Error, Response};
fn main() -> iced::Result {
    iced::application("PatchLite", App::update, App::view).run()
}

#[derive(Default)]
struct App {
    theme: iced::Theme,
    url: String,
    method: Option<HttpMethod>,
    request_body: Option<String>,
    request_headers: Vec<(String, String)>,
    response_message: Option<String>,
    response_message_offset: String,
    request: HttpRequest,
}

#[derive(Debug, Clone)]
enum Message {
    UpdateUrl(String),
    SendRequest,
    UpdateMethod(HttpMethod),
    UpdateAuth(Auth),
    Scrolled(Viewport),
    RequestCompleted(Result<String, String>),
    Clear,
}

#[derive(Default)]
struct HttpMethodComponent {
    method: HttpMethod,
    color: Color,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateUrl(new_url) => {
                self.request.url = new_url;
            }
            Message::SendRequest => {
                if self.request.url.is_empty() {
                    println!("URL is empty!");
                }

                //https://consultafundo.com.br/api/v1/data?taxPayerId=34.430.477/0001-29
                println!("Sending request to {}", self.request.url);

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
        let app_view = column![
            row![
                pick_list(method_pick_list, self.request.method, Message::UpdateMethod,)
                    .placeholder("Select Method"),
                text_input("", self.request.url.as_str()).on_input(|s| Message::UpdateUrl(s)),
                button("Send").on_press(Message::SendRequest),
            ]
            .spacing(10)
            .padding(10),
            horizontal_rule(50),
            row![text("Authentication:")].padding(10),
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
            text("Response will be shown here."),
            Space::with_height(20),
            column![
                Scrollable::new(response)
                    .width(1000)
                    .height(1000)
                    .direction(Direction::Vertical(Scrollbar::new()))
                    .on_scroll(Message::Scrolled),
                text(&self.response_message_offset),
            ]
            .spacing(20),
            row![button("Clear").on_press(Message::Clear),]
        ];

        //let container = Self::container().padding(20).spacing(10).push(app_view);
        app_view.into()
    }
}

fn theme(state: &App) -> Theme {
    Theme::TokyoNight
}
