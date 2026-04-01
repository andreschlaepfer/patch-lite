#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod collection;
mod json_highlight;
mod request;
mod undo;

use crate::collection::{Collection, SavedRequest};
use crate::request::{Auth, HttpMethod, HttpRequest};
use crate::undo::UndoStack;
use iced::keyboard;
use iced::widget::{
    button, column, container, horizontal_rule, pick_list, radio, row, scrollable, text,
    text_editor,
    text_editor::{Action, Content},
    text_input,
};
use iced::{Font, Length, Subscription, Task, Theme};

fn main() -> iced::Result {
    iced::application("PatchLite", App::update, App::view)
        .theme(|_| Theme::TokyoNight)
        .subscription(App::subscription)
        .run_with(App::new)
}

struct App {
    request: HttpRequest,
    request_headers: Vec<(String, String)>,
    request_body_content: Content,
    body_undo: UndoStack,
    tab: Tab,
    response_status: Option<String>,
    response_body: Option<String>,
    response_content: Content,
    // Collections
    collection: Collection,
    save_name: String,
    selected_request: Option<usize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            request: HttpRequest::default(),
            request_headers: Vec::new(),
            request_body_content: Content::new(),
            body_undo: UndoStack::new(),
            tab: Tab::None,
            response_status: None,
            response_body: None,
            response_content: Content::new(),
            collection: Collection::default(),
            save_name: String::new(),
            selected_request: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Init,
    UpdateUrl(String),
    SendRequest,
    UpdateMethod(HttpMethod),
    UpdateAuth(Auth),
    RequestCompleted(Result<(String, String), String>),
    #[allow(dead_code)]
    Clear,
    UpdateBody(Action),
    UpdateTab(Tab),
    UpdateUsername(String),
    UpdatePassword(String),
    UpdateToken(String),
    UpdateHeaderKey(usize, String),
    UpdateHeaderValue(usize, String),
    RemoveHeaderRow(usize),
    AddHeaderRow,
    ResponseAction(Action),
    BodyUndo,
    BodyRedo,
    // Collections
    SaveRequest,
    LoadRequest(usize),
    DeleteRequest(usize),
    UpdateSaveName(String),
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

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();
        app.request.set_default_headers();
        app.request_headers = app
            .request
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
            .collect();
        app.collection = collection::load_collection();
        let task = Task::perform(async {}, |_| Message::Init);
        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Init => {}
            Message::UpdateUrl(new_url) => {
                self.request.url = new_url;
            }
            Message::SendRequest => {
                if self.request.url.is_empty() {
                    return Task::none();
                }

                self.request.set_headers(&self.request_headers);

                let req = self.request.clone();
                return Task::perform(
                    async move {
                        match req.send().await {
                            Ok(response) => {
                                let status = format!("{}", response.status());
                                let body = response.text().await.unwrap_or_default();
                                Ok((status, body))
                            }
                            Err(e) => Err(format!("Request failed: {}", e)),
                        }
                    },
                    Message::RequestCompleted,
                );
            }
            Message::RequestCompleted(result) => match result {
                Ok((status, body)) => {
                    self.response_status = Some(status);
                    let pretty = json_highlight::pretty_json_str(&body);
                    self.response_content = Content::with_text(&pretty);
                    self.response_body = Some(body);
                }
                Err(e) => {
                    self.response_status = Some("Error".to_string());
                    self.response_content = Content::with_text(&e);
                    self.response_body = Some(e);
                }
            },
            Message::ResponseAction(action) => {
                if !action.is_edit() {
                    self.response_content.perform(action);
                }
            }
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
                if let Action::Edit(ref edit) = action {
                    use iced::widget::text_editor::Edit;
                    let kind = match edit {
                        Edit::Insert(c) if c.is_whitespace() || c.is_ascii_punctuation() => {
                            undo::EditKind::InsertBreak
                        }
                        Edit::Insert(_) => undo::EditKind::Insert,
                        Edit::Paste(_) => undo::EditKind::Paste,
                        Edit::Enter => undo::EditKind::Enter,
                        Edit::Backspace | Edit::Delete => undo::EditKind::Delete,
                    };
                    let current = self.request_body_content.text().to_string();
                    self.body_undo.on_before_edit(&current, kind);
                }
                self.request_body_content.perform(action);
                self.request.body = Some(self.request_body_content.text().to_string());
            }
            Message::BodyUndo => {
                let current = self.request_body_content.text().to_string();
                if let Some(prev) = self.body_undo.undo(&current) {
                    self.request_body_content = Content::with_text(prev);
                    self.request.body = Some(self.request_body_content.text().to_string());
                    self.body_undo.pop_undo();
                }
            }
            Message::BodyRedo => {
                let current = self.request_body_content.text().to_string();
                if let Some(next) = self.body_undo.redo(&current) {
                    self.request_body_content = Content::with_text(next);
                    self.request.body = Some(self.request_body_content.text().to_string());
                    self.body_undo.pop_redo();
                }
            }
            Message::UpdateHeaderKey(i, key) => {
                if let Some(header) = self.request_headers.get_mut(i) {
                    header.0 = key;
                }
            }
            Message::UpdateHeaderValue(i, value) => {
                if let Some(header) = self.request_headers.get_mut(i) {
                    header.1 = value;
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
            Message::Clear => {
                self.response_status = None;
                self.response_body = None;
                self.request = HttpRequest::default();
                self.request_headers.clear();
                self.request_body_content = Content::new();
                self.body_undo.clear();
                self.selected_request = None;
            }
            // Collections
            Message::SaveRequest => {
                let name = self.save_name.trim().to_string();
                if !name.is_empty() {
                    let body_text = self.request_body_content.text().to_string();
                    let body = if body_text.trim().is_empty() {
                        None
                    } else {
                        Some(body_text.as_str())
                    };
                    let saved = SavedRequest::from_current(
                        &name,
                        self.request.method,
                        &self.request.url,
                        body,
                        self.request.auth,
                        &self.request.username,
                        &self.request.password,
                        &self.request.token,
                        &self.request_headers,
                    );
                    self.collection.requests.push(saved);
                    collection::save_collection(&self.collection);
                    self.save_name.clear();
                }
            }
            Message::LoadRequest(i) => {
                if let Some(saved) = self.collection.requests.get(i).cloned() {
                    self.request.method = saved.method();
                    self.request.url = saved.url.clone();
                    self.request.auth = saved.auth();
                    self.request.username = saved.username.clone();
                    self.request.password = saved.password.clone();
                    self.request.token = saved.token.clone();
                    self.request_headers = saved.headers.clone();
                    if let Some(body) = &saved.body {
                        self.request_body_content = Content::with_text(body);
                        self.request.body = Some(body.clone());
                    } else {
                        self.request_body_content = Content::new();
                        self.request.body = None;
                    }
                    self.selected_request = Some(i);
                    self.body_undo.clear();
                }
            }
            Message::DeleteRequest(i) => {
                if i < self.collection.requests.len() {
                    self.collection.requests.remove(i);
                    collection::save_collection(&self.collection);
                    if self.selected_request == Some(i) {
                        self.selected_request = None;
                    } else if let Some(sel) = self.selected_request {
                        if sel > i {
                            self.selected_request = Some(sel - 1);
                        }
                    }
                }
            }
            Message::UpdateSaveName(name) => {
                self.save_name = name;
            }
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let main = self.view_main();

        row![sidebar, main]
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn view_sidebar(&self) -> iced::Element<'_, Message> {
        let request_list: Vec<iced::Element<'_, Message>> = self
            .collection
            .requests
            .iter()
            .enumerate()
            .map(|(i, req)| {
                let method_color = HttpMethod::from_str(&req.method)
                    .map(|m| m.color())
                    .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

                // Colored left border accent
                let accent_bar = container(text(""))
                    .width(3)
                    .height(Length::Fixed(20.0))
                    .style(move |_: &Theme| container::Style {
                        background: Some(iced::Background::Color(method_color)),
                        border: iced::Border::default().rounded(1),
                        ..Default::default()
                    });

                let method_text = text(req.method.as_str())
                    .size(11)
                    .font(Font::MONOSPACE);
                let name_text = text(req.name.as_str()).size(13);

                let card_content = row![accent_bar, method_text, name_text]
                    .spacing(6)
                    .align_y(iced::Alignment::Center);

                let entry = button(card_content)
                    .on_press(Message::LoadRequest(i))
                    .width(Length::Fill)
                    .padding([4, 6]);

                let delete_btn = button(text("x").size(11))
                    .on_press(Message::DeleteRequest(i))
                    .padding([2, 6]);

                row![entry, delete_btn]
                    .spacing(2)
                    .align_y(iced::Alignment::Center)
                    .into()
            })
            .collect();

        container(
            column![
                text("Collections").size(16).font(Font::MONOSPACE),
                horizontal_rule(1),
                scrollable(column(request_list).spacing(2))
                    .height(Length::Fill),
                horizontal_rule(1),
                text_input("Request name...", &self.save_name)
                    .on_input(Message::UpdateSaveName)
                    .size(13),
                button(
                    text("Save Current")
                        .size(13)
                        .align_x(iced::alignment::Horizontal::Center),
                )
                .on_press(Message::SaveRequest)
                .width(Length::Fill),
            ]
            .spacing(8)
            .padding(10),
        )
        .style(container::rounded_box)
        .width(220)
        .height(Length::Fill)
        .into()
    }

    fn view_main(&self) -> iced::Element<'_, Message> {
        let method_pick_list = [
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
        ];

        // URL bar — with colored accent for the selected method
        let method_color = self
            .request
            .method
            .map(|m| m.color())
            .unwrap_or(iced::Color::from_rgb(0.4, 0.4, 0.4));

        let method_indicator = container(text(""))
            .width(4)
            .height(Length::Fixed(28.0))
            .style(move |_: &Theme| container::Style {
                background: Some(iced::Background::Color(method_color)),
                border: iced::Border::default().rounded(2),
                ..Default::default()
            });

        let url_bar = container(
            row![
                method_indicator,
                pick_list(method_pick_list, self.request.method, Message::UpdateMethod)
                    .placeholder("Method"),
                text_input("Enter URL...", self.request.url.as_str())
                    .on_input(Message::UpdateUrl),
                button(text("Send").size(14)).on_press(Message::SendRequest),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .padding(10);

        // Tab bar
        let tab_bar = container(
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
            .spacing(10),
        )
        .padding([5, 10]);

        let mut content = column![url_bar, horizontal_rule(1), tab_bar, horizontal_rule(1),]
            .spacing(0)
            .width(Length::Fill);

        // Tab content
        match self.tab {
            Tab::None => {}
            Tab::Auth => {
                let auth_content = container(
                    column![
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
                        .spacing(10),
                    ]
                    .spacing(10),
                )
                .padding(10);

                content = content.push(auth_content);

                match self.request.auth {
                    Auth::Basic => {
                        content = content.push(
                            container(
                                column![
                                    text_input("Username", self.request.username.as_str())
                                        .on_input(Message::UpdateUsername),
                                    text_input("Password", self.request.password.as_str())
                                        .on_input(Message::UpdatePassword),
                                ]
                                .spacing(8),
                            )
                            .padding(10),
                        );
                    }
                    Auth::Bearer => {
                        content = content.push(
                            container(
                                column![text_input("Token", self.request.token.as_str())
                                    .on_input(Message::UpdateToken),]
                                .spacing(8),
                            )
                            .padding(10),
                        );
                    }
                    Auth::None => {}
                }
                content = content.push(horizontal_rule(1));
            }
            Tab::Headers => {
                let mut headers_col = column![
                    row![
                        text("Key").size(13).width(Length::Fill),
                        text("Value").size(13).width(Length::Fill),
                        button(text("+ Add").size(12)).on_press(Message::AddHeaderRow),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                ]
                .spacing(6);

                for (i, (key, value)) in self.request_headers.iter().enumerate() {
                    headers_col = headers_col.push(
                        row![
                            text_input("Key", key.as_str())
                                .on_input(move |k| Message::UpdateHeaderKey(i, k))
                                .size(13),
                            text_input("Value", value.as_str())
                                .on_input(move |v| Message::UpdateHeaderValue(i, v))
                                .size(13),
                            button(text("x").size(12)).on_press(Message::RemoveHeaderRow(i)),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                    );
                }

                content = content.push(container(headers_col).padding(10));
                content = content.push(horizontal_rule(1));
            }
            Tab::Body => {
                content = content.push(
                    container(
                        text_editor(&self.request_body_content)
                            .placeholder("Request body (JSON)...")
                            .on_action(Message::UpdateBody)
                            .height(Length::Fixed(200.0))
                            .font(Font::MONOSPACE)
                            .size(13),
                    )
                    .padding(10),
                );
                content = content.push(horizontal_rule(1));
            }
        }

        // Response area
        let response_widget = self.view_response();
        content = content.push(response_widget);

        content.height(Length::Fill).into()
    }

    fn subscription(_state: &Self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| {
            if !modifiers.command() {
                return None;
            }
            match key {
                keyboard::Key::Character(c) if c.as_str() == "z" => {
                    if modifiers.shift() {
                        Some(Message::BodyRedo)
                    } else {
                        Some(Message::BodyUndo)
                    }
                }
                keyboard::Key::Character(c) if c.as_str() == "y" => Some(Message::BodyRedo),
                _ => None,
            }
        })
    }

    fn view_response(&self) -> iced::Element<'_, Message> {
        match (&self.response_status, &self.response_body) {
            (Some(status), Some(_body)) => {
                let status_bar = container(
                    text(format!("Status: {}", status))
                        .font(Font::MONOSPACE)
                        .size(13),
                )
                .padding([6, 10]);

                let body_widget: iced::Element<'_, Message> =
                    text_editor(&self.response_content)
                        .font(Font::MONOSPACE)
                        .size(13)
                        .highlight_with::<json_highlight::JsonHighlighter>(
                            json_highlight::JsonHighlighterSettings,
                            |highlight, _theme| highlight.to_format(),
                        )
                        .on_action(Message::ResponseAction)
                        .into();

                container(
                    column![status_bar, horizontal_rule(1), body_widget,]
                        .spacing(0)
                        .height(Length::Fill),
                )
                .style(container::rounded_box)
                .padding(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            _ => container(
                text("Send a request to see the response here.")
                    .size(13)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            )
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into(),
        }
    }
}
