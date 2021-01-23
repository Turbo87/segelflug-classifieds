#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

use anyhow::{Context, Result};
use clap::Clap;
use rand::Rng;
use regex::Regex;
use rss::Channel;
use std::collections::HashSet;
use tokio::time::{sleep, Duration};

mod guids;

const FEED_URL: &'static str = "https://www.segelflug.de/osclass/index.php?page=search&sFeed=rss";

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
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let opts: Opts = Opts::parse();
    debug!("parsed options: {:#?}", opts);
    if opts.min_time > opts.max_time {
        let description = String::from("--min-time must not be larger than --max-time");
        clap::Error::with_description(description, clap::ErrorKind::ValueValidation).exit();
    }

    if opts.watch {
        loop {
            run().await?;

            let mins = rand::thread_rng().gen_range(opts.min_time..opts.max_time);
            debug!("running again in {:.1} minutes…", mins);
            let secs = mins * 60.;
            sleep(Duration::from_secs_f32(secs)).await;
        }
    } else {
        run().await?;
    }

    Ok(())
}

async fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    debug!("running in {:?}", cwd);

    let guids_path = cwd.join("last_guids.json");
    let mut guids = guids::read_guids_file(&guids_path).unwrap_or_default();
    trace!("guids = {:#?}", guids);

    debug!("downloading RSS feed from {}", FEED_URL);
    let response = reqwest::get(FEED_URL)
        .await
        .context("Failed to download RSS feed")?;

    let bytes = response
        .bytes()
        .await
        .context("Failed to read response bytes")?;

    debug!("parsing response as RSS feed");
    let channel =
        Channel::read_from(&bytes[..]).context("Failed to parse HTTP response as RSS feed")?;

    let items: Vec<_> = channel
        .items
        .iter()
        .filter(|it| it.guid.is_some() && it.title.is_some())
        .rev()
        .collect();

    debug!("found {} items in the RSS feed", items.len());

    let new_items: Vec<_> = items
        .iter()
        .filter(|it| !guids.contains(&it.guid.as_ref().unwrap().value))
        .collect();

    let new_items = &new_items[..3]; // TODO

    info!("found {} new items in the RSS feed", new_items.len());

    for (index, item) in new_items.iter().enumerate() {
        let title = item.title.as_ref().unwrap();
        info!("- [{}/{}] {}", index + 1, new_items.len(), title);

        if let Some(description) = &item.description {
            info!("{:?}", find_image_url(description));
        }

        let guid = item.guid.as_ref().unwrap();
        guids.insert(guid.value.clone());
    }

    guids::write_guids_file(&guids_path, &guids)?;
    Ok(())
}

fn sanitize_description(value: &str) -> String {
    const LENGTH_LIMIT: usize = 3500;

    // strip HTML tags
    let text = ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(value)
        .to_string();

    // replace HTML entities (only &nbsp; for now...)
    let text = text.replace("&nbsp;", " ");

    // trim surrounding whitespace
    let text = text.trim();

    // limit to `LENGTH_LIMIT` characters
    if text.len() < LENGTH_LIMIT {
        text.to_string()
    } else {
        format!("{}…", &text[..LENGTH_LIMIT - 1])
    }
}

fn find_image_url(description: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#" src="([^"]+)""#).unwrap();
    }

    RE.captures(description)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str())
}
