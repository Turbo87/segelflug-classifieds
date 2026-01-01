use crate::classifieds::rss::parse_feed;
use crate::classifieds::{ClassifiedsDetails, ClassifiedsItem, ClassifiedsUser};
use anyhow::Context;
use reqwest::Client;
use sentry::integrations::anyhow::capture_anyhow;
use tracing::Level;

pub struct ClassifiedsApi {
    client: Client,
    feed_urls: Vec<&'static str>,
}

impl ClassifiedsApi {
    pub fn new(feed_urls: Vec<&'static str>, client: Client) -> Self {
        ClassifiedsApi { client, feed_urls }
    }

    #[instrument(skip(self))]
    pub async fn load_feeds(&self) -> Vec<ClassifiedsItem> {
        let mut items = Vec::new();
        for feed_url in &self.feed_urls {
            let feed_items = match self.load_feed(feed_url).await {
                Ok(items) => items,
                Err(error) => {
                    event!(Level::WARN, "Failed to load feed: {error}");
                    continue;
                }
            };

            items.extend(feed_items.into_iter().filter_map(|result| result.ok()));
        }

        items
    }

    #[instrument(skip(self))]
    async fn load_feed(
        &self,
        feed_url: &str,
    ) -> anyhow::Result<Vec<anyhow::Result<ClassifiedsItem>>> {
        debug!("downloading RSS feed from {feed_url}");
        let response = self.client.get(feed_url).send().await;
        let response = response
            .context("Failed to download RSS feed")?
            .error_for_status()
            .context("Failed to download RSS feed")?;

        let bytes = response.bytes().await;
        let bytes = bytes.context("Failed to read response bytes")?;

        debug!("parsing response as RSS feed");
        let parse_result =
            parse_feed(&bytes[..]).context("Failed to parse HTTP response as RSS feed");
        if let Err(error) = parse_result.as_ref() {
            sentry::with_scope(
                |scope| scope.set_level(Some(sentry::Level::Warning)),
                || capture_anyhow(error),
            );
        }

        parse_result
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
