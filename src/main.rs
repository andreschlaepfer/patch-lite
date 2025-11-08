use std::vec;
mod json_highlight;
use iced::application::View;
use iced::{
    Background, Color, Font,
    font::Family,
    widget::{
        PickList, Rule, Scrollable, Space, TextInput, button, column, horizontal_rule, pick_list,
        row,
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
    response_message: Option<String>,
    response_message_offset: String,
}

#[derive(Debug, Clone)]
enum Message {
    Noop,
    UpdateUrl(String),
    SendRequest,
    UpdateMethod(HttpMethod),
    Scrolled(Viewport),
    RequestCompleted(Result<String, String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
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

#[derive(Default)]
struct HttpMethodComponent {
    method: HttpMethod,
    color: Color,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Noop => {}
            Message::UpdateUrl(new_url) => {
                self.url = new_url;
            }
            Message::SendRequest => {
                if self.url.is_empty() {
                    println!("URL is empty!");
                }

                //https://consultafundo.com.br/api/v1/data?taxPayerId=34.430.477/0001-29
                println!("Sending request to {}", self.url);
                let self_url = self.url.clone();
                return Task::perform(
                    async {
                        let api_client = reqwest::Client::new();
                        let result: Result<Response, Error> = api_client.get(self_url).send().await;

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
                self.method = Some(new_method);
            }
            Message::Scrolled(v) => {
                self.response_message_offset =
                    format!("{} {}", v.absolute_offset().x, v.absolute_offset().y)
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

        column![
            row![
                pick_list(method_pick_list, self.method, Message::UpdateMethod,)
                    .placeholder("Select Method"),
                text_input("", self.url.as_str()).on_input(|s| Message::UpdateUrl(s)),
                button("Send").on_press(Message::SendRequest),
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
            .spacing(20)
        ]
        .into()
    }
}

fn theme(state: &App) -> Theme {
    Theme::TokyoNight
}
