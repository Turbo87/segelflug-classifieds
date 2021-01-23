#[macro_use]
extern crate tracing;
#[macro_use]
extern crate lazy_static;

use crate::price::get_price;
use anyhow::{Context, Result};
use clap::Clap;
use rand::Rng;
use rss::Channel;
use tokio::time::{sleep, Duration};

mod descriptions;
mod guids;
mod price;

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
    trace!("opts = {:#?}", opts);
    if opts.min_time > opts.max_time {
        let description = String::from("--min-time must not be larger than --max-time");
        clap::Error::with_description(description, clap::ErrorKind::ValueValidation).exit();
    }

    if opts.watch {
        loop {
            run().await?;

            let mins = rand::thread_rng().gen_range(opts.min_time..opts.max_time);
            println!("⏳  Running again in {:.1} minutes", mins);
            println!();
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
        .filter(|it| it.guid.is_some() && it.title.is_some() && it.link.is_some())
        .rev()
        .collect();

    debug!("found {} items in the RSS feed", items.len());

    let new_items: Vec<_> = items
        .iter()
        .filter(|it| !guids.contains(&it.guid.as_ref().unwrap().value))
        .collect();

    let new_items = &new_items[..3]; // TODO

    println!(
        "✈️  Found {} new classifieds on Segelflug.de",
        new_items.len()
    );
    println!();

    for item in new_items.iter() {
        let title = item.title.as_ref().unwrap();
        let link = item.link.as_ref().unwrap();
        let price = get_price(link).await?;

        println!(" - {}", title);
        println!("   💶  {}", price);
        println!("   {}", link);
        println!();

        let guid = item.guid.as_ref().unwrap();
        guids.insert(guid.value.clone());
    }

    guids::write_guids_file(&guids_path, &guids)?;
    Ok(())
}
