use super::Generator;
use crate::classifieds::utils::strip_html;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub struct ClassifiedsDetails {
    pub location: Option<String>,
    pub photo_urls: Vec<String>,
    pub price: Option<String>,
    pub user_link: Option<String>,
}

impl From<&str> for ClassifiedsDetails {
    #[instrument(name = "ClassifiedsDetails::from", skip(text))]
    fn from(text: &str) -> Self {
        let html = Html::parse_document(text);
        if !html.errors.is_empty() {
            debug!("found HTML parsing errors: {:?}", html.errors);
        }

        match Generator::detect(&html) {
            Some(Generator::Osclass) => parse_osclass(&html),
            _ => parse_dj_classifieds(&html),
        }
    }
}

fn parse_osclass(html: &Html) -> ClassifiedsDetails {
    lazy_static! {
        static ref ICON_SELECTOR: Selector = Selector::parse(".fa-money").unwrap();
        static ref PHOTO_LINKS_SELECTOR: Selector =
            Selector::parse(".item-photos .thumbs a").unwrap();
        static ref PUB_PROFILE_SELECTOR: Selector =
            Selector::parse("a[href*=\"action=pub_profile\"]").unwrap();
        static ref LOCATION_SELECTOR: Selector = Selector::parse("#item_location").unwrap();
    }

    let price = html
        .select(&ICON_SELECTOR)
        .next()
        .and_then(|icon_element| icon_element.parent())
        .and_then(ElementRef::wrap)
        .map(|price_element| price_element.inner_html())
        .map(|price_html| strip_html(&price_html))
        .map(|price_text| price_text.replace("Euro €", "€").trim().to_string());

    let photo_urls = html
        .select(&PHOTO_LINKS_SELECTOR)
        .filter_map(|element| element.value().attr("href"))
        .filter(|src| !src.ends_with("/no_photo.gif"))
        .map(|src| src.to_string())
        .collect();

    let user_link = html
        .select(&PUB_PROFILE_SELECTOR)
        .next()
        .and_then(|link_element| link_element.value().attr("href"))
        .map(|link| link.to_string());

    let location = html
        .select(&LOCATION_SELECTOR)
        .next()
        .map(|element| element.inner_html())
        .map(|html| strip_html(&html).trim().to_string());

    ClassifiedsDetails {
        location,
        photo_urls,
        price,
        user_link,
    }
}

fn parse_dj_classifieds(html: &Html) -> ClassifiedsDetails {
    lazy_static! {
        static ref PRICE_VAL_SELECTOR: Selector = Selector::parse(".price_val").unwrap();
        static ref PRICE_UNIT_SELECTOR: Selector = Selector::parse(".price_unit").unwrap();
        static ref PHOTO_SELECTOR: Selector =
            Selector::parse(".uk-slideshow-items .el-item img.el-image").unwrap();
        static ref PROFILE_LINK_SELECTOR: Selector =
            Selector::parse("a[href*=\"/component/djclassifieds/profile/\"]").unwrap();
        static ref LOCATION_ICON_SELECTOR: Selector =
            Selector::parse("span[uk-icon=\"icon: location;\"]").unwrap();
        static ref REGION_SELECTOR: Selector = Selector::parse(".reg_path").unwrap();
    }

    let price = html.select(&PRICE_VAL_SELECTOR).next().map(|val_el| {
        let val = val_el.text().collect::<String>();

        html.select(&PRICE_UNIT_SELECTOR)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|unit| format!("{} {unit}", &val))
            .unwrap_or(val)
    });

    let photo_urls = html
        .select(&PHOTO_SELECTOR)
        .filter_map(|el| el.value().attr("src"))
        .map(|src| src.to_string())
        .collect();

    let user_link = html
        .select(&PROFILE_LINK_SELECTOR)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|href| {
            if href.starts_with('/') {
                format!("https://www.segelflug.de{}", href)
            } else {
                href.to_string()
            }
        });

    let location = extract_dj_location(html);

    ClassifiedsDetails {
        location,
        photo_urls,
        price,
        user_link,
    }
}

fn extract_dj_location(html: &Html) -> Option<String> {
    lazy_static! {
        static ref WORLD_ICON_SELECTOR: Selector =
            Selector::parse("span[uk-icon=\"icon: world;\"]").unwrap();
        static ref LOCATION_ICON_SELECTOR: Selector =
            Selector::parse("span[uk-icon=\"icon: location;\"]").unwrap();
        static ref REGION_SELECTOR: Selector = Selector::parse(".reg_path").unwrap();
        static ref CONTENT_SELECTOR: Selector = Selector::parse(".el-content").unwrap();
    }

    // Find location by looking for the location icon, then finding the content div
    let location = html
        .select(&LOCATION_ICON_SELECTOR)
        .next()
        .and_then(|icon| {
            icon.parent()
                .and_then(|p| p.parent())
                .and_then(ElementRef::wrap)
        })
        .map(|el| {
            strip_html(&el.inner_html())
                .trim()
                .trim_start_matches("Flugplatz:")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty() && s != "not available");

    // Find region by looking for the world icon, then finding the content div
    let region = html
        .select(&WORLD_ICON_SELECTOR)
        .next()
        .and_then(|icon| {
            icon.parent()
                .and_then(|p| p.parent())
                .and_then(ElementRef::wrap)
        })
        .map(|el| {
            strip_html(&el.inner_html())
                .trim()
                .trim_start_matches("Europa, ")
                .to_string()
        })
        .filter(|s| !s.is_empty() && s != "Antarktis, Antarktis");

    match (location, region) {
        (Some(location), Some(region)) => Some(format!("{location}, {region}")),
        (Some(location), None) => Some(location),
        (None, Some(region)) => Some(region),
        (None, None) => None,
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
