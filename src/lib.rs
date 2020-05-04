use iced::{
    Align, Application, button, Button, Checkbox, Column, Command, Element, Image, Length, ProgressBar,
    Row, slider, Slider, Text,
};
use iced_native::{Color, Subscription};
use iced_native::widget::image::Handle;
use log::{info, warn};

use crate::chive::Chiver;
use crate::scheduler::PhotoScheduler;

pub mod scheduler;
pub mod chive;

pub struct IcedChive {
    save_btn_state: button::State,
    speed_state: slider::State,
    auto_play: bool,
    play_next: bool,
    speed: f32,
    curr_progress: f32,
    image_url: Option<String>,
    image_data: Option<Vec<u8>>,
    error: Option<String>,
}

impl Application for IcedChive {
    type Executor = iced_futures::executor::Tokio;
    type Message = ChiveMsg;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            IcedChive {
                save_btn_state: button::State::new(),
                speed_state: slider::State::new(),
                auto_play: false,
                play_next: true,
                speed: 5f32,
                curr_progress: 0f32,
                image_url: None,
                image_data: None,
                error: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Hello World!")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use iced_native::input::keyboard::Event as KeyEvent;
        use iced_native::input::mouse::Button;
        use iced_native::input::mouse::Event as MouseEvent;
        use iced_native::input::ButtonState;
        use iced_native::Event;

        match message {
            ChiveMsg::AutoPlay(auto_play) => self.auto_play = auto_play,
            ChiveMsg::Speed(speed) => self.speed = speed,
            ChiveMsg::NativeEvent(Event::Keyboard(KeyEvent::CharacterReceived(_k))) => {
                self.play_next = true;
            }
            ChiveMsg::NativeEvent(Event::Mouse(MouseEvent::Input { button: Button::Right, state: ButtonState::Released })) => {
                self.play_next = true;
            }
            ChiveMsg::Download(download) => match download {
                Download::Progress(curr) => self.curr_progress = curr,
                Download::Error(msg) => {
                    self.error = Some(msg);
                    self.play_next = false;
                }
                Download::Done(data, url) => {
                    self.error = None;
                    self.play_next = false;
                    self.curr_progress = 30f32;
                    self.image_url = Some(url);
                    self.image_data = Some(data);
                }
            },
            ChiveMsg::Save => {
                if let (Some(name), Some(content)) = (&self.image_url, &self.image_data) {
                    let name = &name[name.rfind('/').map(|idx| idx + 1).unwrap_or(0)..];
                    info!("Saving file: {}", name);
                    let dir = std::env::var("HOME")
                        .map(|home| home + "/Pictures")
                        .unwrap_or_else(|_| String::from("."));
                    std::fs::write(format!("{}/{}", dir, name), content)
                        .unwrap_or_else(|e| warn!("Failed to save the image: {:?}", e));
                } else {
                    warn!("Couldn't save the Photo.");
                }
            }
            _ => { /* Ignore all other events */ }
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        use iced_native::subscription;

        let scheduler = PhotoScheduler::new(self.auto_play, self.play_next, self.speed);
        let subs = vec![
            subscription::events().map(ChiveMsg::NativeEvent),
            Subscription::from_recipe(Chiver::default()).map(ChiveMsg::Download),
            Subscription::from_recipe(scheduler).map(|_| ChiveMsg::None),
        ];
        Subscription::batch(subs)
    }

    fn view(&mut self) -> Element<Self::Message> {
        let mut save = Button::new(&mut self.save_btn_state, Text::new("Save"));
        let main_text = if let Some(err) = &self.error {
            Text::new(err).color(Color::from_rgb8(255, 0, 0))
        } else {
            Text::new(self.image_url.as_ref().unwrap_or(&String::default()))
        };
        let image = if let Some(data) = &self.image_data {
            save = save.on_press(ChiveMsg::Save);
            Image::new(Handle::from_memory(data.clone()))
        } else {
            Image::new("matrix.gif")
        };

        Column::new()
            .align_items(Align::Center)
            .padding(10)
            .width(Length::Fill)
            .push(
                Row::new()
                    .padding(10)
                    .spacing(15)
                    .align_items(Align::Center)
                    .push(main_text.height(Length::Units(22)))
                    .push(save),
            )
            .push(
                Row::new()
                    .height(Length::Fill)
                    .align_items(Align::Center)
                    .push(image.width(Length::Fill).height(Length::Fill)),
            )
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .padding(10)
                    .spacing(25)
                    .push(Checkbox::new(self.auto_play, "AutoPlay", ChiveMsg::AutoPlay))
                    .push(
                        Row::new()
                            .align_items(Align::Center)
                            .max_width(200)
                            .spacing(5)
                            .push(Text::new("Speed"))
                            .push(Slider::new(&mut self.speed_state, 1f32..=10f32, self.speed, ChiveMsg::Speed)),
                    )
                    .push(ProgressBar::new(0f32..=30f32, self.curr_progress).height(Length::Units(7))),
            )
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum ChiveMsg {
    None,
    Save,
    AutoPlay(bool),
    Speed(f32),
    NativeEvent(iced_native::Event),
    Download(Download),
}

#[derive(Debug, Clone)]
pub enum Download {
    Progress(f32),
    Error(String),
    Done(Vec<u8>, String),
}
