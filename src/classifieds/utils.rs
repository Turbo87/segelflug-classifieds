use std::collections::HashSet;

pub fn strip_html(value: &str) -> String {
    ammonia::Builder::new()
        .tags(HashSet::new())
        .clean(value)
        .to_string()
}
