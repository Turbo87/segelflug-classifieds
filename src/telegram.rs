use anyhow::{anyhow, Context};
use reqwest::Client;
use teloxide::prelude::*;
use teloxide::requests::Output;
use teloxide::types::{
    InputFile, LinkPreviewOptions, MessageId, ParseMode, Recipient, ReplyParameters,
};
use teloxide::RequestError;
use tokio::time::sleep;

#[derive(Debug)]
pub struct TelegramApi {
    bot: Bot,
    client: Client,
    recipient: Recipient,
}

impl TelegramApi {
    pub fn new<S: Into<String>>(token: S, recipient: S, client: Client) -> Self {
        let bot = Bot::with_client(token, client.clone());

        TelegramApi {
            bot,
            client,
            recipient: Recipient::ChannelUsername(recipient.into()),
        }
    }

    #[instrument(skip(self))]
    pub async fn send_message(
        &self,
        text: &str,
        reply_to: Option<MessageId>,
    ) -> anyhow::Result<Message> {
        let preview_options = LinkPreviewOptions {
            is_disabled: true,
            url: None,
            prefer_small_media: false,
            prefer_large_media: false,
            show_above_text: false,
        };

        let mut request = self
            .bot
            .send_message(self.recipient.clone(), text)
            .parse_mode(ParseMode::Html)
            .link_preview_options(preview_options);

        if let Some(message_id) = reply_to {
            let reply_parameters = ReplyParameters {
                message_id,
                ..Default::default()
            };

            request = request.reply_parameters(reply_parameters);
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
            .send_photo(self.recipient.clone(), InputFile::memory(data))
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
                    debug!("retrying in {} seconds", retry_after.seconds());
                    sleep(retry_after.duration()).await;
                }
                Err(error) => {
                    return Err(error.into());
                }
            };
        }

        Err(anyhow!("Maximum number of retries reached"))
    }
}
