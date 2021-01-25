use crate::classifieds::{strip_html, ClassifiedsApi};
use scraper::{Html, Selector};

pub struct ClassifiedsUser {
    pub name: Option<String>,
    pub address: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

impl ClassifiedsUser {
    pub async fn from_url(url: &str, api: &ClassifiedsApi) -> anyhow::Result<ClassifiedsUser> {
        api.load_user(url).await
    }
}

impl From<&str> for ClassifiedsUser {
    fn from(text: &str) -> Self {
        lazy_static! {
            static ref NAME_SELECTOR: Selector = Selector::parse("li.name").unwrap();
            static ref ADDRESS_SELECTOR: Selector = Selector::parse("li.address").unwrap();
            static ref LOCATION_SELECTOR: Selector = Selector::parse("li.location").unwrap();
            static ref WEBSITE_SELECTOR: Selector = Selector::parse("li.website").unwrap();
        }

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

        Self {
            name,
            address,
            location,
            website,
        }
    }
}
