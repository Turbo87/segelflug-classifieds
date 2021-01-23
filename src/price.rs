use crate::descriptions::strip_html;
use anyhow::{anyhow, Context, Result};
use scraper::{Html, Selector};
use selectors::Element;

pub async fn get_price(url: &str) -> Result<String> {
    lazy_static! {
        static ref ICON_SELECTOR: Selector = Selector::parse(".fa-money").unwrap();
    }

    debug!("downloading HTML file from {}", url);
    let response = reqwest::get(url)
        .await
        .context("Failed to download HTML file")?;

    let text = response
        .text()
        .await
        .context("Failed to read response text")?;

    trace!("text = {:?}", text);

    let html = Html::parse_document(&text);
    if !html.errors.is_empty() {
        debug!("found HTML parsing errors: {:?}", html.errors);
    }

    html.select(&ICON_SELECTOR)
        .next()
        .and_then(|icon_element| icon_element.parent_element())
        .map(|price_element| price_element.inner_html())
        .map(|price_html| strip_html(&price_html))
        .map(|price_text| price_text.replace("Euro €", "€").trim().to_string())
        .ok_or_else(|| anyhow!("Failed to find price on {}", url))
}
