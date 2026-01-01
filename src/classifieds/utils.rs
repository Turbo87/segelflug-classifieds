use std::collections::HashSet;

pub fn strip_html(value: &str) -> String {
    ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(value)
        .to_string()
}

pub fn sanitize_description(value: &str) -> String {
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
        let mut string = text.chars().take(LENGTH_LIMIT).collect::<String>();
        string.push('…');
        string
    }
}
