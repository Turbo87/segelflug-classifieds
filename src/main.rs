#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

use crate::classifieds::ClassifiedsApi;
use crate::telegram::TelegramApi;
use anyhow::Result;
use clap::Clap;
use rand::Rng;
use tokio::time::{sleep, Duration};

mod classifieds;
mod descriptions;
mod guids;
mod telegram;

const FEED_URL: &str = "https://www.segelflug.de/osclass/index.php?page=search&sFeed=rss";

#[derive(Clap, Debug)]
struct Opts {
    /// Run continuously and poll the server in random intervals
    #[clap(short, long)]
    watch: bool,

    /// Minimum time to wait between server requests (in minutes)
    #[clap(long, default_value = "10")]
    min_time: f32,

    /// Maximum time to wait between server requests (in minutes)
    #[clap(long, default_value = "30")]
    max_time: f32,

    /// Telegram chat ID
    #[clap(
        long,
        env = "TELEGRAM_CHAT_ID",
        default_value = "@segelflug_classifieds"
    )]
    telegram_chat_id: String,

    /// Telegram bot token
    #[clap(long, env = "TELEGRAM_TOKEN", hide_env_values = true)]
    telegram_token: Option<String>,
}

struct App {
    classifieds: ClassifiedsApi,
    telegram: Option<TelegramApi>,
}

impl App {
    async fn run(&self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        debug!("running in {:?}", cwd);

        let guids_path = cwd.join("last_guids.json");
        let mut guids = guids::read_guids_file(&guids_path).unwrap_or_default();
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

        guids::write_guids_file(&guids_path, &guids)?;
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let sentry_dsn = std::env::var("SENTRY_DSN");
    let _guard = sentry_dsn.map(sentry::init);

    let opts: Opts = Opts::parse();
    trace!("opts = {:#?}", opts);
    if opts.min_time > opts.max_time {
        let description = String::from("--min-time must not be larger than --max-time");
        clap::Error::with_description(description, clap::ErrorKind::ValueValidation).exit();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let classifieds = ClassifiedsApi::new(FEED_URL, client.clone());

    let telegram = opts
        .telegram_token
        .as_ref()
        .map(|token| TelegramApi::new(token, &opts.telegram_chat_id, client));

    let app = App {
        classifieds,
        telegram,
    };

    if opts.watch {
        app.watch(opts.min_time, opts.max_time).await;
    } else {
        app.run().await?;
    }

    Ok(())
}
