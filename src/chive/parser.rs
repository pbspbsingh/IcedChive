use std::error::Error;

use http::Uri;
use log::{info, warn};
use rand::prelude::*;
use scraper::{Html, Selector};
use serde_json::Value;

const TOTAL_PAGES: i32 = 282;
const CHIVE_URL: &str = "https://thechive.com/category/sexy-girls";

pub fn init_chive_pages() -> Vec<String> {
    info!("Initializing pages from 1 to {}", TOTAL_PAGES);
    let mut pages: Vec<_> = (1..=TOTAL_PAGES).map(|i| if i == 1 {
        format!("{}/", CHIVE_URL)
    } else {
        format!("{}/page/{}/", CHIVE_URL, i)
    }).collect();
    pages.shuffle(&mut thread_rng());
    pages
}

pub fn parse_chive_page<S: AsRef<str>>(html: S) -> Result<Vec<String>, Box<dyn Error>> {
    let document = Html::parse_document(html.as_ref());
    let select =
        Selector::parse("div.main-column div.cards-content div.slot.type-post article.post.card.type-post a.card-img-link[itemprop=image]")
            .unwrap();
    let mut sub_pages = document
        .select(&select)
        .filter_map(|a| a.value().attr("href"))
        .map(String::from)
        .collect::<Vec<_>>();
    sub_pages.shuffle(&mut rand::thread_rng());
    Ok(sub_pages)
}

pub fn parse_chive_sub<S: AsRef<str>>(html: S) -> Result<Vec<String>, Box<dyn Error>> {
    let json = find_json(html.as_ref()).ok_or("No JSON found in Page's content.")?;
    let json = serde_json::from_str::<Value>(json)?;
    let items = json.get("items")
        .ok_or_else(|| format!("No items found in : {}", json))?
        .as_array().ok_or_else(|| format!("Items is not an array: {}", json))?;
    let select = Selector::parse("img").unwrap();
    let mut images = items.iter()
        .map(|item| {
            let html = item["html"].as_str().unwrap_or("<figure />");
            let tp = item["type"].as_str().unwrap_or("attachment");
            let html = Html::parse_fragment(html);
            html.select(&select)
                .filter_map(|img| {
                    if tp == "gif" {
                        img.value().attr("data-gifsrc")
                    } else {
                        img.value().attr("src")
                    }
                })
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .flatten()
        .filter_map(|src| src.parse::<Uri>().ok())
        .map(|link| {
            let scheme = link.scheme().unwrap();
            let host = link.host().unwrap();
            let path = link.path();
            format!("{}://{}{}", scheme, host, path)
        })
        .collect::<Vec<_>>();
    images.shuffle(&mut rand::thread_rng());
    Ok(images)
}

fn find_json(html: &str) -> Option<&str> {
    if let Some(idx) = html.find("CHIVE_GALLERY_ITEMS") {
        let content = &html[idx..];
        if let Some(start) = content.find('{') {
            let mut end = start;
            let mut stack = 0;
            for (i, ch) in content.chars().enumerate() {
                match ch {
                    '{' => stack += 1,
                    '}' => {
                        stack -= 1;
                        if stack == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            return Some(&content[start..=end]);
        }
    }
    warn!("No JSON found:\n{}", html);
    None
}
