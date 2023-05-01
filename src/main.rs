use iced::{Application, Settings};
use crate::ui::Ui;

mod ui;
mod app;
mod server;
mod client;

fn main() -> iced::Result {
    Ui::run(Settings::default())
}