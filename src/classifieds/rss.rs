use crate::classifieds::utils::strip_html;
use ::rss::Item;
use anyhow::anyhow;
use regex::Regex;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ClassifiedsItem {
    pub guid: String,
    pub title: String,
    pub link: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

impl TryFrom<rss::Item> for ClassifiedsItem {
    type Error = anyhow::Error;

    fn try_from(item: Item) -> anyhow::Result<Self, Self::Error> {
        let guid = item.guid;
        let guid = guid.ok_or_else(|| anyhow!("Missing `guid` element"))?.value;

        let title = item.title;
        let title = title.ok_or_else(|| anyhow!("Missing `title` element"))?;

        let link = item.link;
        let link = link.ok_or_else(|| anyhow!("Missing `link` element"))?;

        let description = item.description.as_ref();
        let image_url = description.and_then(|it| find_image_url(&it).map(str::to_string));
        let description = description.map(|it| sanitize_description(&it));

        Ok(ClassifiedsItem {
            guid,
            title,
            link,
            description,
            image_url,
        })
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
