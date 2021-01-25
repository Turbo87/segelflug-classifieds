use crate::classifieds::utils::strip_html;
use crate::classifieds::{ClassifiedsApi, ClassifiedsDetails, ClassifiedsUser};
use ::rss::Item;
use anyhow::anyhow;
use regex::Regex;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct ClassifiedsItem {
    rss_item: rss::Item,
    details: Option<ClassifiedsDetails>,
    user: Option<ClassifiedsUser>,
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

    pub async fn load_details(&mut self, api: &ClassifiedsApi) -> anyhow::Result<()> {
        let link = self.link();
        self.details = Some(api.load_details(link).await?);
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

    pub async fn load_user(&mut self, api: &ClassifiedsApi) -> anyhow::Result<()> {
        assert!(self.can_load_user());
        let user_link = self.user_link().unwrap();

        self.user = Some(api.load_user(user_link).await?);
        Ok(())
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
