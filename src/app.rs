use crate::classifieds::{ClassifiedsApi, ClassifiedsItem};
use crate::guids;
use crate::telegram::TelegramApi;
use anyhow::Result;
use rand::Rng;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

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

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut guids = guids::read_guids_file(&self.guids_path).unwrap_or_default();
        trace!("guids = {:#?}", guids);

        let items = self.classifieds.load_feed().await?;
        let items: Vec<_> = items.into_iter().filter_map(|result| result.ok()).collect();

        debug!("found {} items in the RSS feed", items.len());

        let mut new_items: Vec<_> = items
            .into_iter()
            .rev()
            .filter(|it| !guids.contains(it.guid()))
            .collect();

        println!(
            "✈️  Found {} new classifieds on Segelflug.de",
            new_items.len()
        );
        println!();

        for item in new_items.iter_mut() {
            match self.handle_item(item).await {
                Ok(_) => {
                    guids.insert(item.guid().to_string());
                }
                Err(error) => {
                    warn!("Failed to handle classifieds item: {}", error);
                }
            }
        }

        guids::write_guids_file(&self.guids_path, &guids)?;
        Ok(())
    }

    async fn handle_item(&self, item: &mut ClassifiedsItem) -> Result<()> {
        if let Err(error) = item.load_details(&self.classifieds).await {
            warn!("Failed to load details for {}: {}", item.link(), error);
        }

        if item.can_load_user() {
            if let Err(error) = item.load_user(&self.classifieds).await {
                let user_link = item.user_link().unwrap();
                warn!("Failed to load user details from {}: {}", user_link, error);
            }
        }

        let title = item.title();
        let link = item.link();
        let description = item.description();

        let price = item.details().and_then(|details| details.price.as_ref());

        let user = item.user();
        let user_name = user.and_then(|user| user.name.as_ref());
        let user_location = user.and_then(|user| user.location.as_ref());

        let user_description = match (user_name, user_location) {
            (Some(name), Some(location)) => Some(format!("{} ({})", name, location)),
            (Some(name), None) => Some(name.clone()),
            (None, Some(location)) => Some(location.clone()),
            (None, None) => None,
        };

        let user_emoji = match (user_name, user_location) {
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
            let mut text = format!("<b>{}</b>\n", title);
            if let Some(price) = price {
                text += &format!("<b>💶  {}</b>\n", price);
            }
            if let (Some(user), Some(emoji)) = (&user_description, user_emoji) {
                let user_link = item.user_link().unwrap();
                text += &format!("{}  <a href=\"{}\"><b>{}</b></a>\n", emoji, user_link, user);
            }
            if let Some(description) = description {
                text += &format!("\n{}\n", description);
            }
            text += &format!("\n{}", link);

            telegram.send_message(&text).await?;

            if let Some(image_url) = &item.image_url() {
                if let Err(error) = telegram.send_photo(image_url).await {
                    warn!("Failed to send photo {} to Telegram: {}", image_url, error);
                }
            }
        }

        Ok(())
    }

    pub async fn watch(&self, min_time: f32, max_time: f32) {
        loop {
            if let Err(error) = self.run().await {
                warn!("{}", error);
            }

            let mins = rand::thread_rng().gen_range(min_time..max_time);
            println!("⏳  Running again in {:.1} minutes", mins);
            println!();
            let secs = mins * 60.;
            sleep(Duration::from_secs_f32(secs)).await;
        }
    }
}
