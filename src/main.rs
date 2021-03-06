#[cfg(test)]
#[macro_use]
extern crate insta;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

use crate::app::App;
use crate::classifieds::ClassifiedsApi;
use crate::telegram::TelegramApi;
use anyhow::Result;
use clap::Clap;
use tokio::time::Duration;
use tracing::Level;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::EnvFilter;

mod app;
mod classifieds;
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

#[tokio::main]
async fn main() -> Result<()> {
    Subscriber::builder()
        .pretty()
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let sha = &env!("VERGEN_GIT_SHA")[..7];
    event!(Level::INFO, sha = sha);

    let sentry_dsn = std::env::var("SENTRY_DSN");
    let _guard = sentry_dsn.map(|dsn| {
        let options = sentry::ClientOptions {
            release: Some(sha.into()),
            ..Default::default()
        };
        sentry::init((dsn, options))
    });

    let opts: Opts = Opts::parse();
    event!(Level::DEBUG, opts = ?opts);
    if opts.min_time > opts.max_time {
        let description = String::from("--min-time must not be larger than --max-time");
        clap::Error::with_description(description, clap::ErrorKind::ValueValidation).exit();
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let classifieds = ClassifiedsApi::new(FEED_URL, client.clone());

    let cwd = std::env::current_dir()?;
    event!(Level::INFO, cwd = ?cwd);

    let guids_path = cwd.join("last-guids.json");

    let telegram = opts
        .telegram_token
        .as_ref()
        .map(|token| TelegramApi::new(token, &opts.telegram_chat_id, client));

    let app = App::new(classifieds, guids_path, telegram);
    if opts.watch {
        app.watch(opts.min_time, opts.max_time).await;
    } else {
        app.run().await?;
    }

    Ok(())
}
