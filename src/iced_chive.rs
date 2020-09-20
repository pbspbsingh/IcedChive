use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::time::Duration;

use iced::{Align, Application, Checkbox, Column, Command, Element, Image, Length, Row,
           Slider, slider, Subscription, Text};
use iced_futures::executor;
use iced_native::Event;
use log::*;

use crate::utils::{load_images, read_file};

pub struct IcedChive {
    auto_play: bool,
    speed_state: slider::State,
    speed: f32,
    images: Vec<PathBuf>,
    image: Option<Vec<u8>>,
    title: String,
    dir: String,
}

impl Application for IcedChive {
    type Executor = executor::Tokio;
    type Message = ChiveMessage;
    type Flags = String;

    fn new(dir: Self::Flags) -> (Self, Command<Self::Message>) {
        info!("Initializing the app with flags: {}", dir);
        (
            IcedChive {
                auto_play: true,
                speed_state: slider::State::new(),
                speed: 2.5,
                images: Vec::new(),
                image: None,
                title: String::from("Hello World!"),
                dir: dir.clone(),
            },
            Command::from(load_images(dir)),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
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
            ChiveMessage::ImageData(data) => self.image = Some(data),
            ChiveMessage::NativeEvent(Event::Keyboard(KeyEvent::CharacterReceived(_))) |
            ChiveMessage::NativeEvent(Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Right))) |
            ChiveMessage::Next => {
                let image = self.images.pop();
                image.as_ref().map(|img| img.file_name().map(|file| {
                    self.title = format!("{} ({})", file.to_str().unwrap_or(""), self.images.len());
                }));
                return Command::from(read_file(image));
            }
            _ => {
                // debug!("Ignoring message: {}", message)
            }
        };

        if self.images.is_empty() {
            Command::from(load_images(self.dir.clone()))
        } else {
            Command::none()
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced_native::subscription;
        use iced::time::every;

        let mut subs = vec![subscription::events().map(ChiveMessage::NativeEvent)];
        if self.auto_play {
            subs.push(every(Duration::from_secs_f32(self.speed)).map(|_| {
                debug!("Loading next image...");
                ChiveMessage::Next
            }));
        }
        Subscription::batch(subs)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        use iced_native::widget::image::Handle;

        let image = if let Some(image) = &self.image {
            Image::new(Handle::from_memory(image.clone()))
        } else {
            Image::new("")
        };

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
                            .spacing(5)
                            .push(Text::new("Speed"))
                            .push(Slider::new(
                                &mut self.speed_state,
                                1f32..=10f32,
                                self.speed,
                                ChiveMessage::Speed,
                            )),
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
    ImageData(Vec<u8>),
    NativeEvent(iced_native::Event),
    Next,
}

impl Display for ChiveMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChiveMessage::LoadImages(images) => write!(f, "Images[{}]", images.len()),
            ChiveMessage::ImageData(_) => write!(f, "ImageData"),
            ChiveMessage::NativeEvent(_) => write!(f, "NativeEvent"),
            _ => write!(f, "{:?}", self)
        }
    }
}
