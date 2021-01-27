use crate::classifieds::rss::parse_feed;
use crate::classifieds::{ClassifiedsDetails, ClassifiedsItem, ClassifiedsUser};
use anyhow::Context;
use reqwest::Client;
use tracing::Level;

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

    #[instrument(skip(self))]
    pub async fn load_feed(&self) -> anyhow::Result<Vec<anyhow::Result<ClassifiedsItem>>> {
        debug!("downloading RSS feed from {}", self.feed_url);
        let response = self.client.get(&self.feed_url).send().await;
        let response = response.context("Failed to download RSS feed")?;

        let bytes = response.bytes().await;
        let bytes = bytes.context("Failed to read response bytes")?;

        debug!("parsing response as RSS feed");
        parse_feed(&bytes[..]).context("Failed to parse HTTP response as RSS feed")
    }

    #[instrument(skip(self))]
    pub async fn load_details(&self, url: &str) -> anyhow::Result<ClassifiedsDetails> {
        debug!("loading item details");
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to load item details")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;
        event!(Level::TRACE, text = %text);

        let details = text.as_str().into();
        event!(Level::DEBUG, details = ?details);

        Ok(details)
    }

    #[instrument(skip(self))]
    pub async fn load_user(&self, url: &str) -> anyhow::Result<ClassifiedsUser> {
        debug!("loading user details");
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to download HTML file")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;
        event!(Level::TRACE, text = %text);

        let user = text.as_str().into();
        event!(Level::DEBUG, user = ?user);

        Ok(user)
    }
}
