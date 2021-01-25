use crate::classifieds::{strip_html, ClassifiedsApi};
use scraper::{Html, Selector};
use selectors::Element;

pub struct ClassifiedsDetails {
    pub photo_url: Option<String>,
    pub price: Option<String>,
    pub user_link: Option<String>,
}

impl ClassifiedsDetails {
    pub async fn from_url(url: &str, api: &ClassifiedsApi) -> anyhow::Result<ClassifiedsDetails> {
        api.load_details(url).await
    }
}

impl From<&str> for ClassifiedsDetails {
    fn from(text: &str) -> Self {
        lazy_static! {
            static ref ICON_SELECTOR: Selector = Selector::parse(".fa-money").unwrap();
            static ref PHOTOS_SELECTOR: Selector = Selector::parse(".item-photos img").unwrap();
            static ref PUB_PROFILE_SELECTOR: Selector =
                Selector::parse("a[href*=\"action=pub_profile\"]").unwrap();
        }

        let html = Html::parse_document(text);
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

        Self {
            photo_url,
            price,
            user_link,
        }
    }
}

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
