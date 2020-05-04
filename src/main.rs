use iced::{Application, Settings};

use iced_chive::IcedChive;


pub fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn,iced_chive=debug");
    }
    env_logger::init();

    IcedChive::run(Settings::default());
}