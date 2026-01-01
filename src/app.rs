use crate::classifieds::{ClassifiedsApi, ClassifiedsDetails, ClassifiedsItem, ClassifiedsUser};
use crate::guids;
use crate::telegram::TelegramApi;
use anyhow::Result;
use rand::Rng;
use std::fmt::Write;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::Level;

pub struct App {
    classifieds: ClassifiedsApi,
    guids_path: PathBuf,
    telegram: Option<TelegramApi>,
}

impl App {
    pub fn new(
        classifieds: ClassifiedsApi,
        guids_path: PathBuf,
        telegram: Option<TelegramApi>,
    ) -> Self {
        Self {
            classifieds,
            guids_path,
            telegram,
        }
    }

    #[instrument(skip(self))]
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut guids = guids::read_guids_file(&self.guids_path).unwrap_or_default();
        event!(Level::TRACE, guids = ?guids);

        let items = self.classifieds.load_feeds().await;
        event!(Level::DEBUG, num_items = items.len());

        let new_items: Vec<_> = items
            .into_iter()
            .rev()
            .filter(|it| !guids.contains(&it.guid))
            .collect();

        let num_new_items = new_items.len();
        event!(Level::DEBUG, num_new_items = num_new_items);

        println!("✈️  Found {} new items on Segelflug.de", num_new_items);
        println!();

        for item in new_items.into_iter() {
            match self.handle_item(&item).await {
                Ok(_) => {
                    guids.insert(item.guid);
                }
                Err(error) => {
                    event!(Level::WARN, error = ?error, "Failed to handle classifieds item");
                }
            }
        }

        guids::write_guids_file(&self.guids_path, &guids)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_item(&self, item: &ClassifiedsItem) -> Result<()> {
        let item = self.load_extra_data(item).await?;

        // print item to the console

        println!(" - {}", item.title());
        if let Some(price) = item.price() {
            println!("   💶  {}", price);
        }
        if let (Some(user), Some(emoji)) = (item.user_description(), item.user_emoji()) {
            println!("   {}  {}", emoji, user);
        }
        println!("   {}", item.link());
        println!();

        // send item to Telegram

        if self.telegram.is_some() {
            self.send_item(&item).await?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn load_extra_data<'a>(
        &self,
        item: &'a ClassifiedsItem,
    ) -> Result<ItemWithExtraData<'a>> {
        let link = &item.link;

        let details = match self.classifieds.load_details(link).await {
            Ok(details) => Some(details),
            Err(error) => {
                event!(Level::WARN, error = ?error, "Failed to load details");
                None
            }
        };

        let user_link = details
            .as_ref()
            .and_then(|details| details.user_link.as_ref());
        let user = match user_link.as_ref() {
            Some(user_link) => match self.classifieds.load_user(user_link).await {
                Ok(details) => Some(details),
                Err(error) => {
                    event!(Level::WARN, error = ?error, "Failed to load user details");
                    None
                }
            },
            None => None,
        };

        Ok(ItemWithExtraData {
            item,
            details,
            user,
        })
    }

    async fn send_item(&self, item: &ItemWithExtraData<'_>) -> Result<()> {
        assert!(self.telegram.is_some());
        let telegram = self.telegram.as_ref().unwrap();

        if let Some(photo_url) = item.photo_url() {
            if self.send_item_as_photo(item, photo_url).await.is_ok() {
                return Ok(());
            }
        }

        if let Some(image_url) = item.thumbnail_url() {
            match self.send_item_as_photo(item, image_url).await {
                Ok(_) => return Ok(()),
                Err(error) => {
                    event!(Level::WARN, error = ?error, "Failed to send photo to Telegram");
                }
            };
        }

        telegram.send_message(&item.telegram_text(), None).await?;

        Ok(())
    }

    async fn send_item_as_photo(
        &self,
        item: &ItemWithExtraData<'_>,
        photo_url: &str,
    ) -> Result<()> {
        assert!(self.telegram.is_some());
        let telegram = self.telegram.as_ref().unwrap();

        let telegram_text = item.telegram_text();
        let telegram_short_text = item.telegram_short_text();
        let needs_extra_message = telegram_text.len() > 900;

        let caption = if needs_extra_message {
            &telegram_short_text
        } else {
            &telegram_text
        };

        let message = telegram.send_photo(photo_url, caption).await?;

        if needs_extra_message {
            if let Err(error) = telegram
                .send_message(&telegram_text, Some(message.id))
                .await
            {
                event!(Level::WARN, error = ?error, "Failed to send reply message to Telegram");
            }
        }

        Ok(())
    }

    pub async fn watch(&self, min_time: f32, max_time: f32) {
        loop {
            if let Err(error) = self.run().await {
                event!(Level::WARN, error = ?error);
            }

            let mins = rand::rng().random_range(min_time..max_time);
            println!("⏳  Running again in {:.1} minutes", mins);
            println!();
            let secs = mins * 60.;
            sleep(Duration::from_secs_f32(secs)).await;
        }
    }
}

struct ItemWithExtraData<'a> {
    item: &'a ClassifiedsItem,
    details: Option<ClassifiedsDetails>,
    user: Option<ClassifiedsUser>,
}

impl ItemWithExtraData<'_> {
    pub fn title(&self) -> &str {
        &self.item.title
    }

    pub fn link(&self) -> &str {
        &self.item.link
    }

    pub fn price(&self) -> Option<&str> {
        self.details
            .as_ref()
            .and_then(|details| details.price.as_deref())
    }

    pub fn description(&self) -> Option<&str> {
        self.details
            .as_ref()
            .and_then(|details| details.description.as_deref())
            .or(self.item.description.as_deref())
    }

    pub fn photo_url(&self) -> Option<&str> {
        self.details
            .as_ref()
            .and_then(|details| details.photo_urls.first())
            .map(|it| it.as_str())
    }

    pub fn thumbnail_url(&self) -> Option<&str> {
        self.item.image_url.as_deref()
    }

    pub fn location(&self) -> Option<&str> {
        self.item_location().or_else(|| self.user_location())
    }

    pub fn item_location(&self) -> Option<&str> {
        self.details
            .as_ref()
            .and_then(|details| details.location.as_deref())
    }

    pub fn user_name(&self) -> Option<&str> {
        self.user.as_ref().and_then(|user| user.name.as_deref())
    }

    pub fn user_location(&self) -> Option<&str> {
        self.user.as_ref().and_then(|user| user.location.as_deref())
    }

    pub fn user_link(&self) -> Option<&str> {
        self.details
            .as_ref()
            .and_then(|details| details.user_link.as_deref())
    }
    pub fn user_description(&self) -> Option<String> {
        match (self.user_name(), self.location()) {
            (Some(name), Some(location)) => Some(format!("{} ({})", name, location)),
            (Some(name), None) => Some(name.to_string()),
            (None, Some(location)) => Some(location.to_string()),
            (None, None) => None,
        }
    }

    pub fn user_emoji(&self) -> Option<&str> {
        match (self.user_name(), self.location()) {
            (Some(_), Some(_)) => Some("🧑‍✈️"),
            (Some(_), None) => Some("🧑‍✈️"),
            (None, Some(_)) => Some("🌍"),
            (None, None) => None,
        }
    }

    pub fn telegram_text(&self) -> String {
        self._telegram_text(self.description())
    }

    pub fn telegram_short_text(&self) -> String {
        self._telegram_text(None)
    }

    fn _telegram_text(&self, description: Option<&str>) -> String {
        let mut text = format!("<a href=\"{}\"><b>{}</b></a>\n", self.link(), self.title());

        if let Some(price) = self.price() {
            writeln!(text, "<b>💶  {}</b>", price).unwrap();
        }

        if let (Some(user), Some(emoji)) = (self.user_description(), self.user_emoji()) {
            let user_link = self.user_link().unwrap();
            writeln!(
                text,
                "{}  <a href=\"{}\"><b>{}</b></a>",
                emoji, user_link, user
            )
            .unwrap();
        }

        if let Some(description) = description {
            writeln!(text).unwrap();
            writeln!(text, "{}", description).unwrap();
        }

        writeln!(text).unwrap();
        write!(text, "{}", self.link()).unwrap();

        text
    }
}
