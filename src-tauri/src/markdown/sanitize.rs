use std::collections::HashSet;

use ammonia::{Builder, UrlRelative};

pub fn clean(html: &str) -> String {
    let tags: HashSet<&str> = [
        "a",
        "blockquote",
        "br",
        "code",
        "del",
        "div",
        "em",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "img",
        "li",
        "ol",
        "p",
        "pre",
        "span",
        "strong",
        "table",
        "tbody",
        "td",
        "th",
        "thead",
        "tr",
        "ul",
    ]
    .into_iter()
    .collect();

    let attrs: HashSet<&str> = ["alt", "class", "href", "id", "src", "style", "title"]
        .into_iter()
        .collect();
    let schemes: HashSet<&str> = ["http", "https", "mailto", "data"].into_iter().collect();

    Builder::default()
        .tags(tags)
        .generic_attributes(attrs)
        .url_schemes(schemes)
        .url_relative(UrlRelative::Deny)
        .attribute_filter(|element, attribute, value| match (element, attribute) {
            ("a", "href") if value.starts_with("data:") => None,
            ("img", "src") if !value.starts_with("data:image/") => None,
            _ => Some(value.into()),
        })
        .clean(html)
        .to_string()
}
