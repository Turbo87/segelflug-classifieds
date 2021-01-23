use std::collections::HashSet;

use regex::Regex;

pub fn sanitize_description(value: &str) -> String {
    const LENGTH_LIMIT: usize = 3500;

    // strip HTML tags
    let text = ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(value)
        .to_string();

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

pub fn find_image_url(description: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#" src="([^"]+)""#).unwrap();
    }

    RE.captures(description)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str())
}
