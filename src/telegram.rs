use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

pub struct TelegramApi {
    client: Client,
    chat_id: String,
    token: String,
}

impl TelegramApi {
    pub fn new<S: Into<String>>(token: S, chat_id: S, client: Client) -> Self {
        TelegramApi {
            client,
            chat_id: chat_id.into(),
            token: token.into(),
        }
    }

    pub async fn send_message(&self, text: &str) -> Result<()> {
        let params = serde_urlencoded::to_string(&[
            ("chat_id", self.chat_id.as_str()),
            ("parse_mode", "html"),
            ("disable_web_page_preview", "true"),
            ("text", text),
        ])
        .context("Failed to serialize query parameters")?;

        let action = format!("sendMessage?{}", params);

        self.send_request(&action).await
    }

    pub async fn send_photo(&self, url: &str) -> Result<()> {
        let params =
            serde_urlencoded::to_string(&[("chat_id", self.chat_id.as_str()), ("photo", url)])
                .context("Failed to serialize query parameters")?;

        let action = format!("sendPhoto?{}", params);

        self.send_request(&action).await
    }

    async fn send_request(&self, action: &str) -> Result<()> {
        const NUM_ATTEMPTS: i32 = 5;

        debug!("sending {} to Telegram", action);
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, action);

        for i in 0..NUM_ATTEMPTS {
            if i > 0 {
                debug!("Retrying {} ({}/{})", action, i + 1, NUM_ATTEMPTS);
            }

            let response = self.client.get(&url).send().await?;

            let status = response.status();
            debug!("Telegram responded with {}", status);
            if status.is_success() {
                return Ok(());
            }

            let bytes = response.bytes().await?;
            let json: serde_json::Value = serde_json::from_slice(&bytes)?;
            trace!("json = {}", json);

            let json = json
                .as_object()
                .ok_or_else(|| anyhow!("Unexpected JSON content"))?;

            if status == StatusCode::TOO_MANY_REQUESTS {
                let parameters = json
                    .get("parameters")
                    .and_then(|value| value.as_object())
                    .ok_or_else(|| anyhow!("Unexpected JSON content"))?;

                let retry_after = parameters
                    .get("retry_after")
                    .and_then(|value| value.as_u64())
                    .ok_or_else(|| anyhow!("Unexpected JSON content"))?;

                let retry_after = Duration::from_secs(retry_after);

                sleep(retry_after).await;

                debug!(
                    "Retrying Telegram action in {} seconds",
                    retry_after.as_secs()
                );

                continue;
            } else {
                let description = json
                    .get("description")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| anyhow!("Unexpected JSON content"))?;

                return Err(anyhow!("Telegram API Error: {}", description));
            }
        }

        Err(anyhow!("Maximum number of retries reached"))
    }
}
