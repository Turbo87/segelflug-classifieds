use crate::classifieds::ClassifiedsApi;
use crate::guids;
use crate::telegram::TelegramApi;
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

        let new_items: Vec<_> = items
            .iter()
            .rev()
            .filter(|it| !guids.contains(it.guid()))
            .collect();

        let new_items = &new_items[..1]; // TODO

        println!(
            "✈️  Found {} new classifieds on Segelflug.de",
            new_items.len()
        );
        println!();

        for item in new_items.iter() {
            let title = item.title();
            let link = item.link();
            let price = item.load_price(&self.classifieds).await?;

            println!(" - {}", title);
            println!("   💶  {}", price);
            println!("   {}", link);
            println!();

            let guid = item.guid();
            guids.insert(guid.to_string());
        }

        guids::write_guids_file(&self.guids_path, &guids)?;
        Ok(())
    }

    pub async fn watch(&self, min_time: f32, max_time: f32) -> () {
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
