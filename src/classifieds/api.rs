use crate::classifieds::rss::parse_feed;
use crate::classifieds::{ClassifiedsDetails, ClassifiedsItem, ClassifiedsUser};
use anyhow::Context;
use reqwest::Client;

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

    pub async fn load_feed(&self) -> anyhow::Result<Vec<anyhow::Result<ClassifiedsItem>>> {
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
        parse_feed(&bytes[..]).context("Failed to parse HTTP response as RSS feed")
    }

    pub async fn load_details(&self, url: &str) -> anyhow::Result<ClassifiedsDetails> {
        debug!("downloading HTML file from {}", url);
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to download HTML file")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;

        trace!("text = {:?}", text);

        Ok(text.as_str().into())
    }

    pub async fn load_user(&self, url: &str) -> anyhow::Result<ClassifiedsUser> {
        debug!("downloading HTML file from {}", url);
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to download HTML file")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;

        trace!("text = {:?}", text);

        Ok(text.as_str().into())
    }
}
