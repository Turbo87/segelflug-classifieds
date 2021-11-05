use anyhow::{anyhow, Context};
use reqwest::Client;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::requests::Output;
use teloxide::types::{ChatId, InputFile, ParseMode};
use teloxide::RequestError;
use tokio::time::sleep;

#[derive(Debug)]
pub struct TelegramApi {
    bot: Bot,
    client: Client,
    chat_id: ChatId,
}

impl TelegramApi {
    pub fn new<S: Into<String>>(token: S, chat_id: S, client: Client) -> Self {
        let bot = Bot::with_client(token, client.clone());

        TelegramApi {
            bot,
            client,
            chat_id: ChatId::ChannelUsername(chat_id.into()),
        }
    }

    #[instrument(skip(self))]
    pub async fn send_message(&self, text: &str, reply_to: Option<i32>) -> anyhow::Result<Message> {
        let mut request = self
            .bot
            .send_message(self.chat_id.clone(), text)
            .parse_mode(ParseMode::Html)
            .disable_web_page_preview(true);

        if let Some(message_id) = reply_to {
            request = request.reply_to_message_id(message_id)
        }

        self.send_request(request).await
    }

    #[instrument(skip(self))]
    pub async fn send_photo(&self, url: &str, text: &str) -> anyhow::Result<Message> {
        let request = self.client.get(url);
        let response = request.send().await.context("Failed to request photo")?;
        let bytes = response.bytes().await.context("Failed to download photo")?;
        let data = bytes.as_ref().to_owned();

        let request = self
            .bot
            .send_photo(self.chat_id.clone(), InputFile::memory("photo.jpg", data))
            .caption(text)
            .parse_mode(ParseMode::Html);

        self.send_request(request).await
    }

    #[instrument(skip(self, request))]
    async fn send_request<T>(&self, request: T) -> anyhow::Result<Output<T>>
    where
        T: Request<Err = RequestError>,
    {
        const NUM_ATTEMPTS: i32 = 5;

        debug!("Sending request");

        for i in 0..NUM_ATTEMPTS {
            if i > 0 {
                debug!("Retrying… ({}/{})", i + 1, NUM_ATTEMPTS);
            }

            let response: Result<_, T::Err> = request.send_ref().await;
            match response {
                Ok(response) => return Ok(response),
                Err(RequestError::RetryAfter(retry_after)) => {
                    let retry_after = Duration::from_secs(retry_after as u64);

                    debug!("retrying in {} seconds", retry_after.as_secs());

                    sleep(retry_after).await;
                }
                Err(error) => {
                    return Err(error.into());
                }
            };
        }

        Err(anyhow!("Maximum number of retries reached"))
    }
}
