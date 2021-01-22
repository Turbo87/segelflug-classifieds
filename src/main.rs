use anyhow::Result;
use rss::Channel;

#[tokio::main]
async fn main() -> Result<()> {
    let resp = reqwest::get("https://www.segelflug.de/osclass/index.php?page=search&sFeed=rss")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&resp[..])?;
    println!("{:#?}", channel);
    Ok(())
}
