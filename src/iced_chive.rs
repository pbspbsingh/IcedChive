use std::path::PathBuf;
use std::time::Duration;

use iced::{Align, Application, button, Button, Checkbox, Column, Command, Element, Image, Length, Row, Slider, slider, Subscription, Text};
use iced_futures::executor;
use iced_native::Event;
use log::*;

use crate::utils::load_images;

pub struct IcedChive {
    auto_play: bool,
    speed_state: slider::State,
    speed: f32,
    del_state: button::State,
    images: Vec<PathBuf>,
    image: Option<PathBuf>,
}

impl Application for IcedChive {
    type Executor = executor::Tokio;
    type Message = ChiveMessage;
    type Flags = String;

    fn new(dir: Self::Flags) -> (Self, Command<Self::Message>) {
        info!("Initializing the app with flags: {}", dir);
        (
            IcedChive {
                auto_play: false,
                speed_state: slider::State::new(),
                speed: 20.,
                del_state: button::State::new(),
                images: Vec::new(),
                image: None,
            },
            Command::from(load_images(dir)),
        )
    }

    fn title(&self) -> String {
        if let Some(image) = &self.image {
            return image.file_name()
                .map(|name| format!("[{}] {}", self.images.len(), name.to_str().unwrap_or("")))
                .unwrap_or_else(|| String::from("couldn't get file name"));
        }
        String::from("Hello World!")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use iced_native::keyboard::Event as KeyEvent;
        use iced_native::mouse::Event as MouseEvent;
        use iced::mouse::Button as MouseButton;

        match message {
            ChiveMessage::LoadImages(images) => self.images = images,
            ChiveMessage::Error(msg) => error!("Error: {}", msg),
            ChiveMessage::AutoPlay(auto_play) => self.auto_play = auto_play,
            ChiveMessage::Speed(speed) => self.speed = speed,
            ChiveMessage::Delete => {
                if let Some(img) = &self.image {
                    match trash::remove(img) {
                        Ok(_) => info!("Deleted '{}'", img.to_string_lossy()),
                        Err(e) => error!("Failed to delete '{}': {}", img.to_string_lossy(), e)
                    };
                }
            }
            ChiveMessage::NativeEvent(Event::Keyboard(KeyEvent::CharacterReceived(_))) |
            ChiveMessage::NativeEvent(Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Right))) |
            ChiveMessage::Next => {
                self.image = self.images.pop();
                self.image.as_ref().map(|image| info!("Next image: {:?}", image));
            }
            _ => {
                // debug!("Ignoring message: {}", message)
            }
        };
        if self.images.is_empty() && self.auto_play {
            warn!("Images are empty, can't autoplay!");
            Command::from(async { ChiveMessage::AutoPlay(false) })
        } else {
            Command::none()
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced_native::subscription;
        use iced::time::every;

        let mut subs = vec![subscription::events().map(ChiveMessage::NativeEvent)];
        let wait = self.speed / 10.;
        if self.auto_play {
            subs.push(every(Duration::from_secs_f32(wait)).map(move |_| {
                debug!("Loading next image after waiting {:0.2}s", wait);
                ChiveMessage::Next
            }));
        }
        Subscription::batch(subs)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let image = Image::new(self.image.as_ref().unwrap_or(&PathBuf::new()));

        Column::new()
            .align_items(Align::Center)
            .padding(10)
            .width(Length::Fill)
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .padding(10)
                    .spacing(25)
                    .push(Checkbox::new(self.auto_play, "AutoPlay", ChiveMessage::AutoPlay))
                    .push(
                        Row::new()
                            .align_items(Align::Center)
                            .max_width(250)
                            .spacing(10)
                            .push(Text::new("Speed"))
                            .push(Slider::new(&mut self.speed_state, 3.0..=100., self.speed, ChiveMessage::Speed))
                            .push(Button::new(&mut self.del_state, Text::new("Delete")).on_press(ChiveMessage::Delete)),
                    ),
            )
            .push(
                Row::new()
                    .height(Length::Fill)
                    .align_items(Align::Center)
                    .push(image.width(Length::Fill).height(Length::Fill)),
            )
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum ChiveMessage {
    LoadImages(Vec<PathBuf>),
    Error(String),
    AutoPlay(bool),
    Speed(f32),
    Delete,
    NativeEvent(iced_native::Event),
    Next,
}
