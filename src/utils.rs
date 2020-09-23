use std::collections::VecDeque;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Instant;

use log::{error, info};
use rand::{self, seq::SliceRandom};

use crate::iced_chive::ChiveMessage;

const SUPPORTED_IMAGES: &[&str] = &["jpeg", "jpg", "png", "gif", "bmp"];

pub fn home_dir() -> String {
    env::var("HOME").unwrap_or_else(|_| env::var("HOMEDIR").expect("Failed to get user's home dir"))
}

pub async fn load_images(dir: impl AsRef<str>) -> ChiveMessage {
    info!("Loading images from: {}", dir.as_ref());
    let start = Instant::now();
    match load(dir.as_ref()) {
        Ok(mut images) => {
            info!("Loaded {} images in {} ms", images.len(), start.elapsed().as_millis());
            images.shuffle(&mut rand::thread_rng());
            ChiveMessage::LoadImages(images)
        }
        Err(e) => {
            error!("Image loading failed: {}", e);
            ChiveMessage::Error(e.to_string())
        }
    }
}

fn load(dir: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut images = Vec::new();

    let mut queue = VecDeque::new();
    queue.push_back(PathBuf::from(dir));

    while !queue.is_empty() {
        let pop = queue.pop_front().unwrap();
        if pop.is_file() && pop.extension().map(match_ext).unwrap_or(false) {
            images.push(pop);
        } else if pop.is_dir() {
            for file in pop.read_dir()? {
                let file = file?;
                queue.push_back(file.path());
            }
        }
    }
    Ok(images)
}

fn match_ext(ext: &OsStr) -> bool {
    let ext = ext.to_str().unwrap_or("").to_lowercase();
    SUPPORTED_IMAGES.contains(&(&ext as &str))
}
