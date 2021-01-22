use anyhow::Result;
use rss::Channel;

const FEED_URL: &'static str = "https://www.segelflug.de/osclass/index.php?page=search&sFeed=rss";

#[tokio::main]
async fn main() -> Result<()> {
    let resp = reqwest::get(FEED_URL).await?.bytes().await?;
    let channel = Channel::read_from(&resp[..])?;
    println!("{:#?}", channel);
    Ok(())
}
