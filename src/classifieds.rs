use anyhow::{anyhow, Context, Result};
use regex::Regex;
use reqwest::Client;
use rss::{Channel, Item};
use scraper::{Html, Selector};
use selectors::Element;
use std::collections::HashSet;
use std::convert::TryFrom;

pub struct ClassifiedsItem {
    rss_item: rss::Item,
    details: Option<ClassifiedsDetails>,
    user: Option<ClassifiedsUser>,
}

impl TryFrom<rss::Item> for ClassifiedsItem {
    type Error = anyhow::Error;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        if item.guid.is_none() {
            return Err(anyhow!("Missing `guid` element"));
        }
        if item.title.is_none() {
            return Err(anyhow!("Missing `title` element"));
        }
        if item.link.is_none() {
            return Err(anyhow!("Missing `link` element"));
        }

        Ok(ClassifiedsItem {
            rss_item: item,
            details: None,
            user: None,
        })
    }
}

impl ClassifiedsItem {
    pub fn guid(&self) -> &str {
        &self.rss_item.guid.as_ref().unwrap().value
    }

    pub fn title(&self) -> &str {
        &self.rss_item.title.as_ref().unwrap()
    }

    pub fn link(&self) -> &str {
        &self.rss_item.link.as_ref().unwrap()
    }

    pub fn description(&self) -> Option<String> {
        let description = self.rss_item.description.as_ref();
        description.map(|it| sanitize_description(&it))
    }

    pub fn image_url(&self) -> Option<String> {
        let description = self.rss_item.description.as_ref();
        description.and_then(|it| find_image_url(&it).map(str::to_string))
    }

    pub fn details(&self) -> Option<&ClassifiedsDetails> {
        self.details.as_ref()
    }

    pub async fn load_details(&mut self, api: &ClassifiedsApi) -> Result<()> {
        let link = self.link();
        self.details = Some(ClassifiedsDetails::from_url(link, api).await?);
        Ok(())
    }

    pub fn user_link(&self) -> Option<&String> {
        self.details()
            .and_then(|details| details.user_link.as_ref())
    }

    pub fn can_load_user(&self) -> bool {
        self.user_link().is_some()
    }

    pub fn user(&self) -> Option<&ClassifiedsUser> {
        self.user.as_ref()
    }

    pub async fn load_user(&mut self, api: &ClassifiedsApi) -> Result<()> {
        assert!(self.can_load_user());
        let user_link = self.user_link().unwrap();

        self.user = Some(ClassifiedsUser::from_url(user_link, api).await?);
        Ok(())
    }
}

pub struct ClassifiedsDetails {
    pub photo_url: Option<String>,
    pub price: Option<String>,
    pub user_link: Option<String>,
}

impl ClassifiedsDetails {
    pub async fn from_url(url: &str, api: &ClassifiedsApi) -> Result<ClassifiedsDetails> {
        api.load_details(url).await
    }
}

pub struct ClassifiedsUser {
    pub name: Option<String>,
    pub address: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

impl ClassifiedsUser {
    pub async fn from_url(url: &str, api: &ClassifiedsApi) -> Result<ClassifiedsUser> {
        api.load_user(url).await
    }
}

pub struct ClassifiedsApi {
    client: Client,
    feed_url: String,
}

impl ClassifiedsApi {
    pub fn new<S: Into<String>>(feed_url: S, client: Client) -> Self {
        ClassifiedsApi {
            client,
            feed_url: feed_url.into(),
        }
    }

    pub async fn load_feed(&self) -> Result<Vec<Result<ClassifiedsItem>>> {
        debug!("downloading RSS feed from {}", self.feed_url);
        let response = self
            .client
            .get(&self.feed_url)
            .send()
            .await
            .context("Failed to download RSS feed")?;

        let bytes = response
            .bytes()
            .await
            .context("Failed to read response bytes")?;

        debug!("parsing response as RSS feed");
        let channel =
            Channel::read_from(&bytes[..]).context("Failed to parse HTTP response as RSS feed")?;

        let items = channel
            .items
            .into_iter()
            .map(ClassifiedsItem::try_from)
            .collect();

        Ok(items)
    }

    pub async fn load_details(&self, url: &str) -> Result<ClassifiedsDetails> {
        lazy_static! {
            static ref ICON_SELECTOR: Selector = Selector::parse(".fa-money").unwrap();
            static ref PHOTOS_SELECTOR: Selector = Selector::parse(".item-photos img").unwrap();
            static ref PUB_PROFILE_SELECTOR: Selector =
                Selector::parse("a[href*=\"action=pub_profile\"]").unwrap();
        }

        debug!("downloading HTML file from {}", url);
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to download HTML file")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;

        trace!("text = {:?}", text);

        let html = Html::parse_document(&text);
        if !html.errors.is_empty() {
            debug!("found HTML parsing errors: {:?}", html.errors);
        }

        let price = html
            .select(&ICON_SELECTOR)
            .next()
            .and_then(|icon_element| icon_element.parent_element())
            .map(|price_element| price_element.inner_html())
            .map(|price_html| strip_html(&price_html))
            .map(|price_text| price_text.replace("Euro €", "€").trim().to_string());
        debug!("price = {:?}", price);

        let photo_url = html
            .select(&PHOTOS_SELECTOR)
            .next()
            .and_then(|element| element.value().attr("src"))
            .map(|src| src.to_string());
        debug!("photo_url = {:?}", photo_url);

        let user_link = html
            .select(&PUB_PROFILE_SELECTOR)
            .next()
            .and_then(|link_element| link_element.value().attr("href"))
            .map(|link| link.to_string());
        debug!("user_link = {:?}", user_link);

        Ok(ClassifiedsDetails {
            photo_url,
            price,
            user_link,
        })
    }

    pub async fn load_user(&self, url: &str) -> Result<ClassifiedsUser> {
        lazy_static! {
            static ref NAME_SELECTOR: Selector = Selector::parse("li.name").unwrap();
            static ref ADDRESS_SELECTOR: Selector = Selector::parse("li.address").unwrap();
            static ref LOCATION_SELECTOR: Selector = Selector::parse("li.location").unwrap();
            static ref WEBSITE_SELECTOR: Selector = Selector::parse("li.website").unwrap();
        }

        debug!("downloading HTML file from {}", url);
        let response = self.client.get(url).send().await;
        let response = response.context("Failed to download HTML file")?;

        let text = response.text().await;
        let text = text.context("Failed to read response text")?;

        trace!("text = {:?}", text);

        let html = Html::parse_document(&text);
        if !html.errors.is_empty() {
            debug!("found HTML parsing errors: {:?}", html.errors);
        }

        let name = html
            .select(&NAME_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.trim().to_string());
        debug!("name = {:?}", name);

        let address = html
            .select(&ADDRESS_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.replace("Adresse:", ""))
            .map(|text| text.trim().to_string());
        debug!("address = {:?}", address);

        let location = html
            .select(&LOCATION_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.replace("Standort:", ""))
            .map(|text| text.trim().to_string());
        debug!("location = {:?}", location);

        let website = html
            .select(&WEBSITE_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.trim().to_string());
        debug!("website = {:?}", website);

        Ok(ClassifiedsUser {
            name,
            address,
            location,
            website,
        })
    }
}

fn strip_html(value: &str) -> String {
    ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(value)
        .to_string()
}

fn sanitize_description(value: &str) -> String {
    const LENGTH_LIMIT: usize = 3500;

    // strip HTML tags
    let text = strip_html(value);

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
