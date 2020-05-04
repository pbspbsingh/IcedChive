use std::error::Error;
use std::hash::Hasher;
use std::time::Instant;

use iced_futures::BoxStream;
use iced_futures::futures::stream::{self, StreamExt};
use iced_futures::subscription::Recipe;
use log::{info, warn};
use reqwest::Client;

use crate::chive::parser::{init_chive_pages, parse_chive_page, parse_chive_sub};
use crate::Download;
use crate::scheduler::SCHEDULING_CHANNEL;

mod parser;


const USER_AGENT_VAL: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.14; rv:68.0) Gecko/20100101 Firefox/68.0";

#[derive(Debug)]
enum State {
    None,
    Page,
    SubPage,
    Image,
}

#[derive(Debug)]
pub struct Chiver {
    pages: Vec<String>,
    sub_pages: Vec<String>,
    images: Vec<String>,
    client: Option<Client>,
    is_running: bool,
    state: State,
}

impl Chiver {
    pub fn default() -> Self {
        Chiver {
            pages: Vec::new(),
            sub_pages: Vec::new(),
            images: Vec::new(),
            client: None,
            is_running: false,
            state: State::None,
        }
    }

    fn client(&mut self) -> &Client {
        use reqwest::header::*;
        use reqwest::Proxy;

        if self.client.is_none() {
            let use_proxy = !std::env::args().any(|arg| arg.starts_with("--noproxy"));
            info!("Initializing http client, using proxy: {}", use_proxy);

            let headers = vec![(USER_AGENT, USER_AGENT_VAL.parse().unwrap())]
                .into_iter()
                .collect();
            let mut client = Client::builder()
                .cookie_store(true)
                .default_headers(headers);
            if use_proxy {
                let proxy = Proxy::all("socks5://127.0.0.1:9150").expect("Invalid proxy provided");
                client = client.proxy(proxy);
            }
            self.client = Some(client.build().expect("Failed to create http client."));
        }
        self.client.as_ref().unwrap()
    }

    async fn download(&mut self) -> Result<Download, Box<dyn Error>> {
        let start = Instant::now();
        match self.state {
            State::None => {
                if self.pages.is_empty() {
                    self.pages = init_chive_pages();
                }
                self.state = State::Page;
                Ok(Download::Progress(1f32))
            }
            State::Page => {
                if self.sub_pages.is_empty() {
                    let page = self.pages.pop().ok_or("Pages is empty!")?;
                    let page_content = self.client().get(&page).send().await?.text().await?;
                    self.sub_pages = parse_chive_page(page_content)?;
                    info!("Time: {}, SubPages: {}, Page: {}", start.elapsed().as_millis(), self.sub_pages.len(), page);
                }
                self.state = State::SubPage;
                Ok(Download::Progress(10f32))
            }
            State::SubPage => {
                if self.images.is_empty() {
                    let sub_page = self.sub_pages.pop().ok_or("SubPages is empty!")?;
                    let sub_page_content = self.client().get(&sub_page).send().await?.text().await?;
                    self.images = parse_chive_sub(sub_page_content)?;
                    info!("Time: {}, Images: {}, SubPage: {}", start.elapsed().as_millis(), self.images.len(), sub_page);
                }
                self.state = State::Image;
                Ok(Download::Progress(20f32))
            }
            State::Image => {
                let image = self.images.pop().ok_or("Images is empty!")?;
                let img_res = self.client().get(&image).send().await?;
                if img_res.status() != 200 {
                    return Err(format!("Http status: {}, {}", img_res.status(), image).into());
                }
                let bytes = img_res.bytes().await?.to_vec();
                info!("Time: {}, Image: {}", start.elapsed().as_millis(), image);
                self.state = State::None;
                Ok(Download::Done(bytes, image))
            }
        }
    }
}

impl<H: Hasher, E> Recipe<H, E> for Chiver {
    type Output = Download;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<E>) -> BoxStream<Self::Output> {
        stream::unfold(self, |mut state| async move {
            if !state.is_running {
                let mut receiever = SCHEDULING_CHANNEL.1.lock().await;
                receiever.recv().await.unwrap();
                receiever.recv().await.unwrap();

                info!("Received command to download photo.");
                state.is_running = true;
                state.state = State::None;
            }
            let status = match state.download().await {
                Err(e) => {
                    warn!("Failed to download: {}", e.to_string());
                    Download::Error(e.to_string())
                }
                Ok(status) => status
            };
            if matches!(&status, Download::Error(_) | Download::Done(_, _)) {
                state.is_running = false;
                info!("Pages: {}, SubPages: {}, Images: {}", state.pages.len(), state.sub_pages.len(), state.images.len());
            }
            Some((status, state))
        }).boxed()
    }
}