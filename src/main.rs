use std::vec;

use iced::Theme;
use iced::application::View;
use iced::{
    Background, Color, Font,
    font::Family,
    widget::{
        PickList, Rule, Scrollable, TextInput, button, column, horizontal_rule, pick_list, row,
        scrollable::{Direction, Scrollbar, Viewport},
        text, text_input,
        text_input::{Icon, Side},
    },
};

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
    UpdateUrl(String),
    SendRequest,
    UpdateMethod(HttpMethod),
    Scrolled(Viewport),
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

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::UpdateUrl(new_url) => {
                self.url = new_url;
            }
            Message::SendRequest => {
                if self.url.is_empty() {
                    println!("URL is empty!");
                    return;
                }
                println!("Sending request to {}", self.url);
                self.response_message = String::from(
                    "[
    {
        \"id\": 74532628,
        \"dataPatrimonioLiquido\": \"2025-09-30T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-10-15T00:00:00.000Z\",
        \"numeroContas\": 2959877,
        \"valorPatrimonioLiquido\": 114532364.28
    },
    {
        \"id\": 74051786,
        \"dataPatrimonioLiquido\": \"2025-08-31T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-09-15T00:00:00.000Z\",
        \"numeroContas\": 2959877,
        \"valorPatrimonioLiquido\": 104500880.75
    },
    {
        \"id\": 73497634,
        \"dataPatrimonioLiquido\": \"2025-07-31T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-08-19T00:00:00.000Z\",
        \"numeroContas\": 2959877,
        \"valorPatrimonioLiquido\": 102868164.54
    },
    {
        \"id\": 72859533,
        \"dataPatrimonioLiquido\": \"2025-06-30T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-08-14T00:00:00.000Z\",
        \"numeroContas\": 2959877,
        \"valorPatrimonioLiquido\": 101619621.28
    },
    {
        \"id\": 72664547,
        \"dataPatrimonioLiquido\": \"2025-05-31T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-06-16T00:00:00.000Z\",
        \"numeroContas\": 2927265,
        \"valorPatrimonioLiquido\": 97310294.66
    },
    {
        \"id\": 72664060,
        \"dataPatrimonioLiquido\": \"2025-04-30T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-05-29T00:00:00.000Z\",
        \"numeroContas\": 2889720,
        \"valorPatrimonioLiquido\": 95681744.35
    },
    {
        \"id\": 73969245,
        \"dataPatrimonioLiquido\": \"2025-03-31T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-04-15T00:00:00.000Z\",
        \"numeroContas\": 1444860,
        \"valorPatrimonioLiquido\": 94790355.26
    },
    {
        \"id\": 73969362,
        \"dataPatrimonioLiquido\": \"2025-02-28T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-04-15T00:00:00.000Z\",
        \"numeroContas\": 1444860,
        \"valorPatrimonioLiquido\": 94910848.22
    },
    {
        \"id\": 73573482,
        \"dataPatrimonioLiquido\": \"2025-01-31T00:00:00.000Z\",
        \"dataAtualizacao\": \"2025-02-17T00:00:00.000Z\",
        \"numeroContas\": 1444860,
        \"valorPatrimonioLiquido\": 95019242.14
    }
]",
                )
                .into();
            }
            Message::UpdateMethod(new_method) => {
                self.method = Some(new_method);
            }
            Message::Scrolled(v) => {
                self.response_message_offset =
                    format!("{} {}", v.absolute_offset().x, v.absolute_offset().y)
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let method_pick_list = [
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
        ];

        let response = column(
            (self.response_message.as_deref().unwrap_or(""))
                .split("\n")
                .map(|i| text(format!("{}", i)).into()),
        );

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
            column![
                Scrollable::new(response)
                    .width(1000)
                    .height(1000)
                    .direction(Direction::Vertical(Scrollbar::new()))
                    .on_scroll(Message::Scrolled),
                text(&self.response_message_offset),
            ]
        ]
        .into()
    }
}

fn theme(state: &App) -> Theme {
    Theme::TokyoNight
}
