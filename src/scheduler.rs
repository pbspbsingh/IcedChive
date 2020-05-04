use iced_futures::BoxStream;
use iced_futures::futures::stream;
use iced_futures::futures::StreamExt;
use log::info;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::Duration;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCHEDULING_CHANNEL: (Sender<()>, Mutex<Receiver<()>>) = scheduler_channel();
}

fn scheduler_channel() -> (Sender<()>, Mutex<Receiver<()>>) {
    let (sender, receiver) = mpsc::channel::<()>(1);
    (sender, Mutex::new(receiver))
}

#[derive(Debug)]
pub struct PhotoScheduler {
    auto_play: bool,
    play_next: bool,
    timeout: u64,
}

impl PhotoScheduler {
    pub fn new(auto_play: bool, play_next: bool, timeout: f32) -> Self {
        PhotoScheduler {
            auto_play,
            play_next,
            timeout: (timeout * 1000f32) as u64,
        }
    }
}

impl<H: std::hash::Hasher, E: 'static> iced_native::subscription::Recipe<H, E> for PhotoScheduler {
    type Output = ();

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.auto_play.hash(state);
        self.play_next.hash(state);
        self.timeout.hash(state);
    }

    fn stream(self: Box<Self>, _inputs: BoxStream<E>) -> BoxStream<Self::Output> {
        stream::once(async move {
            let mut sender = SCHEDULING_CHANNEL.0.clone();
            if self.play_next {
                sender.send(()).await.unwrap();
                sender.send(()).await.unwrap();
                info!("Sent a playnext command.");
            }

            #[allow(clippy::while_immutable_condition)]
            while self.auto_play {
                tokio::time::delay_for(Duration::from_millis(self.timeout)).await;
                sender.send(()).await.unwrap();
                sender.send(()).await.unwrap();
                info!("Sent a playnext command after waiting for {}", self.timeout);
            }
        }).boxed()
    }
}
