use crate::classifieds::utils::strip_html;
use scraper::{Html, Selector};
use selectors::Element;

#[derive(Debug)]
pub struct ClassifiedsDetails {
    pub location: Option<String>,
    pub photo_url: Option<String>,
    pub price: Option<String>,
    pub user_link: Option<String>,
}

impl From<&str> for ClassifiedsDetails {
    fn from(text: &str) -> Self {
        lazy_static! {
            static ref ICON_SELECTOR: Selector = Selector::parse(".fa-money").unwrap();
            static ref PHOTOS_SELECTOR: Selector = Selector::parse(".item-photos img").unwrap();
            static ref PUB_PROFILE_SELECTOR: Selector =
                Selector::parse("a[href*=\"action=pub_profile\"]").unwrap();
            static ref LOCATION_SELECTOR: Selector = Selector::parse("#item_location").unwrap();
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

        let location = html
            .select(&LOCATION_SELECTOR)
            .next()
            .map(|element| element.inner_html())
            .map(|html| strip_html(&html).trim().to_string());
        debug!("location = {:?}", location);

        Self {
            location,
            photo_url,
            price,
            user_link,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClassifiedsDetails;
    use std::fs;

    #[test]
    fn parse_test() {
        glob!("test-input/item/*.html", |path| {
            let text = fs::read_to_string(path).unwrap();
            let user = ClassifiedsDetails::from(text.as_str());
            assert_debug_snapshot!(user);
        });
    }
}
