use crate::descriptions::strip_html;
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use rss::Channel;
use scraper::{Html, Selector};
use selectors::Element;

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

    pub async fn load_feed(&self) -> Result<Channel> {
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

        Ok(channel)
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
