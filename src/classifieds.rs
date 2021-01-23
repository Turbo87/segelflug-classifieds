use crate::descriptions::{find_image_url, sanitize_description, strip_html};
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use selectors::Element;
use std::convert::TryFrom;

pub struct ClassifiedsItem {
    item: rss::Item,
}

impl TryFrom<rss::Item> for ClassifiedsItem {
    type Error = anyhow::Error;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        if item.guid.is_none() {
            return Err(anyhow!("Missing `guid` element"));
        }
        if item.title.is_none() {
            return Err(anyhow!("Missing `title` element"));
        }
        if item.link.is_none() {
            return Err(anyhow!("Missing `link` element"));
        }

        Ok(ClassifiedsItem { item })
    }
}

impl ClassifiedsItem {
    pub fn guid(&self) -> &str {
        &self.item.guid.as_ref().unwrap().value
    }

    pub fn title(&self) -> &str {
        &self.item.title.as_ref().unwrap()
    }

    pub fn link(&self) -> &str {
        &self.item.link.as_ref().unwrap()
    }

    pub fn description(&self) -> Option<String> {
        let description = self.item.description.as_ref();
        description.map(|it| sanitize_description(&it))
    }

    pub fn image_url(&self) -> Option<String> {
        let description = self.item.description.as_ref();
        description.and_then(|it| find_image_url(&it).map(str::to_string))
    }

    pub async fn load_price(&self, api: &ClassifiedsApi) -> Result<String> {
        api.load_price(self.link()).await
    }
}

pub struct ClassifiedsApi {
    client: Client,
    feed_url: String,
}

impl ClassifiedsApi {
    pub fn new<S: Into<String>>(feed_url: S, client: Client) -> Self {
        ClassifiedsApi {
            client,
            feed_url: feed_url.into(),
        }
    }

    pub async fn load_feed(&self) -> Result<Vec<Result<ClassifiedsItem>>> {
        debug!("downloading RSS feed from {}", self.feed_url);
        let response = self
            .client
            .get(&self.feed_url)
            .send()
            .await
            .context("Failed to download RSS feed")?;

        let bytes = response
            .bytes()
            .await
            .context("Failed to read response bytes")?;

        debug!("parsing response as RSS feed");
        let channel =
            Channel::read_from(&bytes[..]).context("Failed to parse HTTP response as RSS feed")?;

        let items = channel
            .items
            .into_iter()
            .map(|item| ClassifiedsItem::try_from(item))
            .collect();

        Ok(items)
    }

    pub async fn load_price(&self, url: &str) -> Result<String> {
        lazy_static! {
            static ref ICON_SELECTOR: Selector = Selector::parse(".fa-money").unwrap();
        }

        debug!("downloading HTML file from {}", url);
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to download HTML file")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;

        trace!("text = {:?}", text);

        let html = Html::parse_document(&text);
        if !html.errors.is_empty() {
            debug!("found HTML parsing errors: {:?}", html.errors);
        }

        html.select(&ICON_SELECTOR)
            .next()
            .and_then(|icon_element| icon_element.parent_element())
            .map(|price_element| price_element.inner_html())
            .map(|price_html| strip_html(&price_html))
            .map(|price_text| price_text.replace("Euro €", "€").trim().to_string())
            .ok_or_else(|| anyhow!("Failed to find price on {}", url))
    }
}
