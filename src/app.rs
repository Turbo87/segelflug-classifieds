use crate::classifieds::{ClassifiedsApi, ClassifiedsDetails, ClassifiedsItem};
use crate::guids;
use crate::telegram::TelegramApi;
use anyhow::Result;
use rand::Rng;
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

        let items = self.classifieds.load_feed().await?;
        let items: Vec<_> = items.into_iter().filter_map(|result| result.ok()).collect();
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

        let title = &item.title;
        let description = &item.description;

        let price = details.as_ref().and_then(|details| details.price.as_ref());
        let item_location = details
            .as_ref()
            .and_then(|details| details.location.as_ref());

        let user_name = user.as_ref().and_then(|user| user.name.as_ref());
        let user_location = user.as_ref().and_then(|user| user.location.as_ref());

        let location = item_location.or(user_location);

        let user_description = match (user_name, location) {
            (Some(name), Some(location)) => Some(format!("{} ({})", name, location)),
            (Some(name), None) => Some(name.clone()),
            (None, Some(location)) => Some(location.clone()),
            (None, None) => None,
        };

        let user_emoji = match (user_name, location) {
            (Some(_), Some(_)) => Some("🧑‍✈️"),
            (Some(_), None) => Some("🧑‍✈️"),
            (None, Some(_)) => Some("🌍"),
            (None, None) => None,
        };

        // print item to the console

        println!(" - {}", title);
        if let Some(price) = &price {
            println!("   💶  {}", price);
        }
        if let (Some(user), Some(emoji)) = (&user_description, user_emoji) {
            println!("   {}  {}", emoji, user);
        }
        println!("   {}", link);
        println!();

        // send item to Telegram

        if let Some(telegram) = &self.telegram {
            let mut text = format!("<a href=\"{}\"><b>{}</b></a>\n", link, title);
            if let Some(price) = price {
                text += &format!("<b>💶  {}</b>\n", price);
            }
            if let (Some(user), Some(emoji)) = (&user_description, user_emoji) {
                let user_link = user_link.unwrap();
                text += &format!("{}  <a href=\"{}\"><b>{}</b></a>\n", emoji, user_link, user);
            }
            if let Some(description) = description {
                text += &format!("\n{}\n", description);
            }
            text += &format!("\n{}", link);

            telegram.send_message(&text).await?;

            if let Err(error) = self.send_photo_for_item(item, details.as_ref()).await {
                event!(Level::WARN, error = ?error, "Failed to send photo to Telegram");
            }
        }

        Ok(())
    }

    async fn send_photo_for_item(
        &self,
        item: &ClassifiedsItem,
        details: Option<&ClassifiedsDetails>,
    ) -> Result<()> {
        assert!(self.telegram.is_some());
        let telegram = self.telegram.as_ref().unwrap();

        let photo_url = details.and_then(|details| details.photo_urls.first());
        if let Some(photo_url) = photo_url {
            if telegram.send_photo(photo_url).await.is_ok() {
                return Ok(());
            }
        }

        if let Some(image_url) = &item.image_url {
            telegram.send_photo(image_url).await
        } else {
            Ok(())
        }
    }

    pub async fn watch(&self, min_time: f32, max_time: f32) {
        loop {
            if let Err(error) = self.run().await {
                event!(Level::WARN, error = ?error);
            }

            let mins = rand::thread_rng().gen_range(min_time..max_time);
            println!("⏳  Running again in {:.1} minutes", mins);
            println!();
            let secs = mins * 60.;
            sleep(Duration::from_secs_f32(secs)).await;
        }
    }
}
