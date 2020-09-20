use std::env;

use iced::{Application, Settings};

use crate::iced_chive::IcedChive;
use crate::utils::home_dir;

mod iced_chive;
mod utils;

pub fn main() -> iced::Result {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn,iced_chive=debug");
    }
    env_logger::init();

    let mut settings = Settings::default();
    settings.flags = env::args().nth(1).unwrap_or_else(|| format!("{}/Pictures", home_dir()));
    IcedChive::run(settings)
}
