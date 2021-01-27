use crate::classifieds::utils::strip_html;
use ::rss::Item;
use anyhow::anyhow;
use regex::Regex;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ClassifiedsItem {
    rss_item: rss::Item,
}

impl TryFrom<rss::Item> for ClassifiedsItem {
    type Error = anyhow::Error;

    fn try_from(item: Item) -> anyhow::Result<Self, Self::Error> {
        if item.guid.is_none() {
            return Err(anyhow!("Missing `guid` element"));
        }
        if item.title.is_none() {
            return Err(anyhow!("Missing `title` element"));
        }
        if item.link.is_none() {
            return Err(anyhow!("Missing `link` element"));
        }

        Ok(ClassifiedsItem { rss_item: item })
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
