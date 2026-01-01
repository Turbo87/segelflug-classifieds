use crate::classifieds::utils::strip_html;
use scraper::{ElementRef, Html, Selector};

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
            static ref GENERATOR_SELECTOR: Selector =
                Selector::parse("meta[name=\"generator\"]").unwrap();
        }

        let html = Html::parse_document(text);
        if !html.errors.is_empty() {
            debug!("found HTML parsing errors: {:?}", html.errors);
        }

        let generator = html
            .select(&GENERATOR_SELECTOR)
            .next()
            .and_then(|el| el.value().attr("content"))
            .unwrap_or("");

        if generator.starts_with("Osclass") {
            parse_osclass(&html)
        } else {
            parse_dj_classifieds(&html)
        }
    }
}

fn parse_osclass(html: &Html) -> ClassifiedsUser {
    lazy_static! {
        static ref NAME_SELECTOR: Selector = Selector::parse("li.name").unwrap();
        static ref ADDRESS_SELECTOR: Selector = Selector::parse("li.address").unwrap();
        static ref LOCATION_SELECTOR: Selector = Selector::parse("li.location").unwrap();
        static ref WEBSITE_SELECTOR: Selector = Selector::parse("li.website").unwrap();
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

    ClassifiedsUser {
        name,
        address,
        location,
        website,
    }
}

fn parse_dj_classifieds(html: &Html) -> ClassifiedsUser {
    lazy_static! {
        static ref NAME_SELECTOR: Selector =
            Selector::parse(".djc-profile-box h3.el-title").unwrap();
        static ref STANDORT_HEADER_SELECTOR: Selector = Selector::parse("h2.uk-h4").unwrap();
    }

    let name = html
        .select(&NAME_SELECTOR)
        .next()
        .map(|el| el.text().collect::<String>());

    let location = extract_dj_location(html);

    ClassifiedsUser {
        name,
        address: None,
        location,
        website: None,
    }
}

fn extract_dj_location(html: &Html) -> Option<String> {
    lazy_static! {
        static ref STANDORT_HEADER_SELECTOR: Selector = Selector::parse("h2.uk-h4").unwrap();
    }

    // Find the "Standort" header and extract location from following siblings
    let standort_header = html
        .select(&STANDORT_HEADER_SELECTOR)
        .find(|el| el.text().collect::<String>().contains("Standort"))?;

    let mut location_name: Option<String> = None;
    let mut region: Option<String> = None;

    // Iterate through siblings after the header
    let mut next = standort_header.next_sibling();
    while let Some(sibling) = next {
        if let Some(el) = ElementRef::wrap(sibling) {
            let text = strip_html(&el.inner_html())
                .trim()
                .trim_start_matches("Europa, ")
                .to_string();

            if text.starts_with("Flugplatz:") {
                let loc = text.trim_start_matches("Flugplatz:").trim();
                if !loc.is_empty() && loc != "not available" {
                    location_name = Some(loc.to_string());
                }
            } else if el.select(&Selector::parse(".reg_path").unwrap()).count() > 0 {
                if !text.is_empty() && text != "Antarktis, Antarktis" {
                    region = Some(text);
                }
            } else if el.value().name() == "h2" {
                // Stop at next header
                break;
            }
        }
        next = sibling.next_sibling();
    }

    match (location_name, region) {
        (Some(loc), Some(reg)) => Some(format!("{loc}, {reg}")),
        (Some(loc), None) => Some(loc),
        (None, Some(reg)) => Some(reg),
        (None, None) => None,
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
