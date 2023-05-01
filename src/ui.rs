use iced::{Application, Command, Element, Event, executor, keyboard, Length, mouse, touch, window};
use iced::alignment;
use iced::widget::{button, column, container, row, text, pick_list, image, qr_code, QRCode};
use iced_aw::{Card, Modal};
use iced_aw::selection_list::{SelectionList, SelectionListStyles};
use iced_native::{Subscription, subscription, event};
use crate::app::{App, Screenshot, Screenshots, SteamApp};
use crate::server;
use crate::client;

const DEFAULT_QR_CELL_SIZE: u16 = 10;
const DEFAULT_DIALOG_TEXT_SIZE: u16 = 30;
const DEFAULT_DIALOG_WIDTH: f32 = 450.0;

pub struct Ui {
    show_modal: bool,
    app: App,

    steam_apps: Vec<SteamApp>,
    selected_steam_app: Option<SteamApp>,

    images: Screenshots,
    show_images: Vec<Screenshot>,
    selected_image: Option<Screenshot>,
    selected_image_index: Option<usize>,
    images_focused: bool,
    image_selected: bool,

    display_image: image::Handle,
    display_changed: bool,
    share_url: Option<String>,
    qr_code: Option<qr_code::State>,
    qr_cell_size: u16,
    dialog_width: f32,
    dialog_text_size: u16,
}

#[derive(Debug, Clone)]
pub enum Message {
    First,
    AppSelected(SteamApp),
    ScreenshotSelected(usize),
    Share,
    StopShare,
    KeyUp,
    KeyDown,
    Clicked,
    Server,
    FetchFinished,
}

async fn sleep_for_first() {
    use async_std::task::sleep;
    use std::time::Duration;

    sleep(Duration::from_millis(300)).await;
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    match key_code {
        KeyCode::Up => Some(Message::KeyUp),
        KeyCode::Down => Some(Message::KeyDown),
        _ => None,
    }
}

impl Ui {
    fn update_selected_image_index(&mut self, increment: bool) {
        if self.selected_image_index.is_none() {
            return;
        }

        if self.selected_steam_app.is_none() {
            return;
        }

        let diff: i32 = if increment { 1 } else { -1 };

        let mut index: i32 = self.selected_image_index
            .unwrap() as i32;

        index += diff;

        let max_index: i32 = (self.show_images.len() - 1) as i32;

        if index < 0 {
            index = max_index;
        } else if index > max_index {
            index = 0;
        }

        let new_index: usize = index as usize;

        self.selected_image_index = Some(new_index.clone());
        self.display_changed = !self.update_selected_image(new_index);
    }

    fn update_selected_image(&mut self, index: usize) -> bool{
        let screenshot = self.show_images.get(index).clone().unwrap();
        let image_changed = if self.selected_image.is_some() {
            self.selected_image.clone().unwrap().filepath != screenshot.filepath
        } else {
            true
        };
        let new_image = Some(screenshot.clone());
        self.selected_image = new_image;
        image_changed
    }
}

impl Application for Ui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut app = App::new();
        app.check_env();
        app.setup();

        let all_app = SteamApp {
            id: String::from("0"),
            title: String::from(""),
        };

        let selected_steam_app = Some(all_app.clone());

        let mut steam_apps = Vec::new();
        steam_apps.push(all_app);

        let images = app.get_images();
        let show_images = Vec::new();
        let selected_image = None;
        let selected_image_index = None;

        let dummy_pixels = vec![0u8];
        let display_image = image::Handle::from_pixels(
            1, 1, dummy_pixels,
        );

        let qr_cell_size = ((DEFAULT_QR_CELL_SIZE as f64) * app.scale_factor) as u16;
        let dialog_text_size = ((DEFAULT_DIALOG_TEXT_SIZE as f64) * app.scale_factor) as u16;
        let dialog_width = DEFAULT_DIALOG_WIDTH * (app.scale_factor as f32);

        let startup_cmd = Command::perform(
            sleep_for_first(),
            |_| Message::First,
        );

        let server_port = app.server_port.clone();
        let shared_image = app.shared_image.clone();

        let server_cmd = Command::perform(
            server::run(shared_image, server_port),
            |_| Message::Server,
        );

        let applist_path = app.applist_json_path.clone();

        let fetch_cmd = Command::perform(
            client::fetch_applist(applist_path),
            |_| Message::FetchFinished,
        );

        (
            Ui {
                show_modal: false,
                app,
                steam_apps,
                selected_steam_app,
                images,
                show_images,
                selected_image,
                selected_image_index,
                images_focused: false,
                image_selected: false,
                display_image,
                display_changed: true,
                share_url: None,
                qr_code: None,
                qr_cell_size,
                dialog_text_size,
                dialog_width,
            },
            Command::batch([
                startup_cmd,
                server_cmd,
                fetch_cmd,
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("share screenshot")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::First => {
                return window::resize(2000, 2000);
            },
            Message::FetchFinished => {
                self.app.post_fetch();

                let apps = self.app.get_steam_apps();
                self.steam_apps.extend(apps);

                self.images = self.app.get_images();
                self.show_images.extend(self.images.sorted_all.clone());
            },
            Message::AppSelected(app) => {
                let selected_app = Some(app);
                if self.selected_steam_app == selected_app {
                    return Command::none();
                }
                self.selected_steam_app = selected_app;
                self.selected_image = None;
                self.selected_image_index = None;
                let dummy_pixels = vec![0u8];
                self.display_image = image::Handle::from_pixels(
                    1, 1, dummy_pixels,
                );
                if let Some(selected_app) = self.selected_steam_app.clone() {
                    self.show_images.clear();
                    if selected_app.id == "0" {
                        self.show_images.extend(
                            self.images.sorted_all.clone());
                    } else {
                        self.show_images.extend(
                        self.images.sorted_by_app
                            .get(&selected_app.id)
                            .unwrap()
                            .clone());
                    }
                }
            },
            Message::ScreenshotSelected(index) => {
                self.images_focused = true;
                self.image_selected = true;
                if self.selected_steam_app.is_none() {
                    return Command::none();
                }
                self.selected_image_index = Some(index.clone());
                if self.update_selected_image(index) || !self.display_changed {
                    self.display_image = image::Handle::from_path(
                        self.selected_image.clone().unwrap().filepath);
                    self.display_changed = true;
                }
            },
            Message::Share => {
                if let Some(selected_image) = self.selected_image.clone() {
                    self.share_url = Some(self.app.share(selected_image));
                    self.qr_code = if self.share_url == None {
                        None
                    } else {
                        qr_code::State::new(
                            &self.share_url.clone().unwrap()).ok()
                    };
                    self.show_modal = self.share_url.is_some();
                }
            },
            Message::StopShare => {
                self.show_modal = false;
                self.share_url = None;
                self.app.stop_share();
            },
            Message::KeyUp => {
                if !self.show_modal && self.images_focused {
                    self.update_selected_image_index(false);
                }
            },
            Message::KeyDown => {
                if !self.show_modal && self.images_focused {
                    self.update_selected_image_index(true);
                }
            },
            Message::Clicked => {
                if !self.image_selected {
                    self.images_focused = false;
                }
                self.image_selected = false;
            },
            Message::Server => {
                println!("server closed");
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let apps_list = pick_list(
            &self.steam_apps,
            self.selected_steam_app.clone(),
            Message::AppSelected,
        );

        let image_list = SelectionList::new_with(
            &self.show_images,
            Message::ScreenshotSelected,
            self.selected_image_index.clone(),
            12.0,
            5.0,
            SelectionListStyles::Default,
        );

        let content = container(
            column![
                row![
                    apps_list
                    .width(Length::Fill),
                ],
                container(
                    row![
                        image_list
                        .width(Length::Fixed(150f32)),
                        container(
                            column![
                                row![
                                    button(
                                        text("share")
                                        .horizontal_alignment(alignment::Horizontal::Center))
                                    .on_press(Message::Share)
                                    .width(Length::Fill)
                                ],
                                image::viewer(self.display_image.clone())
                                .width(Length::Fill)
                                .height(Length::Fill),
                            ]
                        )
                        .width(Length::Fill)
                        .height(Length::Fill),
                    ]
                )
                .center_x()
                .center_y()
                .width(Length::Fill)
                .height(Length::Fill),
            ]
            .height(Length::Fill),
        )
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill);

        Modal::new(self.show_modal, content, || {
            let mut body = column![
                text(format!(
                    "Access to: {}",
                    if let Some(url) = self.share_url.clone() {
                        url
                    } else {
                        String::from("Unknown")
                    }))
                .size(self.dialog_text_size),
            ];

            if let Some(qr_code) = self.qr_code.as_ref() {
                body = body.push(QRCode::new(qr_code).cell_size(self.qr_cell_size));
            }

            Card::new(
                text("share"),
                body,
            )
            .foot(
                row![
                    button(text("Cancel")
                        .horizontal_alignment(alignment::Horizontal::Center))
                    .width(Length::Fill)
                    .on_press(Message::StopShare),
                ]
                .width(Length::Fill),
            )
            .width(Length::Fixed(self.dialog_width))
            .on_close(Message::StopShare)
            .into()
        })
        .backdrop(Message::StopShare)
        .on_esc(Message::StopShare)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    return Some(Message::Clicked);
                },
                _ => {},
            }

            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code,
                    ..
                }) => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn scale_factor(&self) -> f64 {
        self.app.scale_factor
    }
}