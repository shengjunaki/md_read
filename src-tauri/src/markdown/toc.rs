use std::collections::HashMap;

use pulldown_cmark::HeadingLevel;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TocItem {
    pub level: u8,
    pub id: String,
    pub text: String,
}

#[derive(Default)]
pub struct Slugger {
    seen: HashMap<String, usize>,
}

impl Slugger {
    pub fn slug(&mut self, text: &str) -> String {
        let mut base = String::new();
        let mut last_dash = false;

        for ch in text.chars().flat_map(char::to_lowercase) {
            if ch.is_ascii_alphanumeric() {
                base.push(ch);
                last_dash = false;
            } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '.' | '/') {
                if !last_dash && !base.is_empty() {
                    base.push('-');
                    last_dash = true;
                }
            } else if !ch.is_ascii() {
                base.push(ch);
                last_dash = false;
            }
        }

        while base.ends_with('-') {
            base.pop();
        }

        if base.is_empty() {
            base.push_str("heading");
        }

        let count = self.seen.entry(base.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            base
        } else {
            format!("{base}-{}", *count)
        }
    }
}

#[cfg(test)]
pub fn collect(markdown: &str, options: pulldown_cmark::Options) -> Vec<TocItem> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};

    let parser = Parser::new_ext(markdown, options);
    let mut items = Vec::new();
    let mut slugger = Slugger::default();
    let mut current: Option<(u8, String)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) if heading_rank(level) <= 4 => {
                current = Some((heading_rank(level), String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, text)) = current.take() {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        let id = slugger.slug(&text);
                        items.push(TocItem { level, id, text });
                    }
                }
            }
            Event::Text(text) | Event::Code(text) | Event::InlineMath(text) => {
                if let Some((_, current_text)) = &mut current {
                    current_text.push_str(&text);
                }
            }
            _ => {}
        }
    }

    items
}

pub fn heading_rank(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::Options;

    use super::collect;

    #[test]
    fn collects_h1_to_h4_only() {
        let items = collect("# A\n\n#### D\n\n##### E\n", Options::empty());

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "a");
        assert_eq!(items[1].level, 4);
    }
}
