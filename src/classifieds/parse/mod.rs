mod item;
mod user;

pub use item::ClassifiedsDetails;
pub use user::ClassifiedsUser;

use scraper::{Html, Selector};

pub enum Generator {
    Osclass,
    DjClassifieds,
}

impl Generator {
    pub fn detect(html: &Html) -> Option<Self> {
        lazy_static! {
            static ref GENERATOR_SELECTOR: Selector =
                Selector::parse("meta[name=\"generator\"]").unwrap();
        }

        let content = html
            .select(&GENERATOR_SELECTOR)
            .next()
            .and_then(|el| el.value().attr("content"))?;

        if content.starts_with("Osclass") {
            Some(Self::Osclass)
        } else if content.starts_with("Joomla") {
            Some(Self::DjClassifieds)
        } else {
            None
        }
    }
}
