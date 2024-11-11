use crate::classifieds::utils::strip_html;
use scraper::{Html, Selector};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ClassifiedsUser {
    pub name: Option<String>,
    pub address: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

impl From<&str> for ClassifiedsUser {
    #[instrument(name = "ClassifiedsUser::from", skip(text))]
    fn from(text: &str) -> Self {
        lazy_static! {
            static ref NAME_SELECTOR: Selector = Selector::parse("li.name").unwrap();
            static ref ADDRESS_SELECTOR: Selector = Selector::parse("li.address").unwrap();
            static ref LOCATION_SELECTOR: Selector = Selector::parse("li.location").unwrap();
            static ref WEBSITE_SELECTOR: Selector = Selector::parse("li.website").unwrap();
        }

        let html = Html::parse_document(text);
        if !html.errors.is_empty() {
            debug!("found HTML parsing errors: {:?}", html.errors);
        }

        let name = html
            .select(&NAME_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.trim().to_string());

        let address = html
            .select(&ADDRESS_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.replace("Adresse:", ""))
            .map(|text| text.trim().to_string());

        let location = html
            .select(&LOCATION_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.replace("Standort:", ""))
            .map(|text| text.trim().to_string());

        let website = html
            .select(&WEBSITE_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html))
            .map(|text| text.trim().to_string());

        Self {
            name,
            address,
            location,
            website,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClassifiedsUser;
    use std::fs;

    #[test]
    fn parse_test() {
        glob!("test-input/user/*.html", |path| {
            let text = fs::read_to_string(path).unwrap();
            let user = ClassifiedsUser::from(text.as_str());
            assert_debug_snapshot!(user);
        });
    }
}
