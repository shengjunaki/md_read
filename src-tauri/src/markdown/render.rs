use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::fs::document::MarkdownSource;
use base64::{engine::general_purpose, Engine};
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;

use super::{
    highlight::{plain_code, Highlighter},
    math, sanitize,
    toc::{self, Slugger, TocItem},
};

#[derive(Debug, Clone, Serialize)]
pub struct RenderedDocument {
    pub title: String,
    pub path: String,
    pub html: String,
    pub toc: Vec<TocItem>,
    pub render_ms: u128,
}

pub fn render_document(source: &MarkdownSource) -> RenderedDocument {
    let started = Instant::now();
    let options = markdown_options();
    let (html, toc) = render_html(&source.content, options, &source.base_dir);
    let html = sanitize::clean(&html);
    let title = toc
        .first()
        .map(|item| item.text.clone())
        .or_else(|| {
            source
                .path
                .file_stem()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Markdown Reader".to_string());

    RenderedDocument {
        title,
        path: source.path.display().to_string(),
        html,
        toc,
        render_ms: started.elapsed().as_millis(),
    }
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}

fn render_html(markdown: &str, options: Options, base_dir: &Path) -> (String, Vec<TocItem>) {
    let parser = Parser::new_ext(markdown, options);
    let mut highlighter = None;
    let mut slugger = Slugger::default();
    let mut toc = Vec::new();
    let mut events = Vec::with_capacity(markdown.len().saturating_div(48).clamp(32, 4096));
    let mut iter = parser.peekable();

    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Heading {
                level,
                id: _,
                classes,
                attrs,
            }) => {
                let mut text = String::new();
                let mut inner = Vec::new();
                while let Some(inner_event) = iter.next() {
                    match inner_event {
                        Event::End(TagEnd::Heading(_)) => break,
                        Event::Text(value) | Event::Code(value) | Event::InlineMath(value) => {
                            text.push_str(&value);
                            inner.push(Event::Text(value));
                        }
                        other => inner.push(other),
                    }
                }

                let id = slugger.slug(text.trim());
                let level_rank = toc::heading_rank(level);
                if level_rank <= 4 {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        toc.push(TocItem {
                            level: level_rank,
                            id: id.clone(),
                            text,
                        });
                    }
                }
                events.push(Event::Start(Tag::Heading {
                    level,
                    id: Some(CowStr::Boxed(id.into_boxed_str())),
                    classes,
                    attrs,
                }));
                events.extend(inner);
                events.push(Event::End(TagEnd::Heading(level)));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let mut code = String::new();
                while let Some(inner_event) = iter.next() {
                    match inner_event {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(value) | Event::Code(value) => code.push_str(&value),
                        _ => {}
                    }
                }

                let language = match kind {
                    CodeBlockKind::Fenced(value) => first_language_token(&value).to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let highlighted = if language.is_empty() {
                    plain_code(&language, &code)
                } else {
                    highlighter
                        .get_or_insert_with(Highlighter::new)
                        .highlight(&language, &code)
                };
                code.clear();
                events.push(Event::Html(CowStr::Boxed(highlighted.into_boxed_str())));
            }
            Event::InlineMath(value) => {
                events.push(Event::Html(CowStr::Boxed(
                    math::render(&value, false).into_boxed_str(),
                )));
            }
            Event::DisplayMath(value) => {
                events.push(Event::Html(CowStr::Boxed(
                    math::render(&value, true).into_boxed_str(),
                )));
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                events.push(Event::Start(Tag::Image {
                    link_type,
                    dest_url: image_src(base_dir, &dest_url),
                    title,
                    id,
                }));
            }
            other => events.push(other),
        }
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    (out, toc)
}

fn first_language_token(info: &str) -> &str {
    info.split_whitespace().next().unwrap_or("")
}

fn image_src(base_dir: &Path, raw: &str) -> CowStr<'static> {
    local_image_data_url(base_dir, raw)
        .map(CowStr::Boxed)
        .unwrap_or_else(|| CowStr::Borrowed(""))
}

fn local_image_data_url(base_dir: &Path, raw: &str) -> Option<Box<str>> {
    if raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("data:")
        || raw.starts_with('#')
    {
        return None;
    }

    let path = normalize_local_path(base_dir, raw)?;
    let metadata = fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > 5 * 1024 * 1024 {
        return None;
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let bytes = fs::read(path).ok()?;
    let encoded = general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{encoded}").into_boxed_str())
}

fn normalize_local_path(base_dir: &Path, raw: &str) -> Option<PathBuf> {
    let without_fragment = raw.split('#').next().unwrap_or(raw);
    let decoded = percent_decode_minimal(without_fragment);
    let candidate = PathBuf::from(decoded.as_ref());
    if candidate.is_absolute() {
        return None;
    }
    base_dir.join(candidate).canonicalize().ok()
}

fn percent_decode_minimal(value: &str) -> Cow<'_, str> {
    if !value.contains('%') {
        return Cow::Borrowed(value);
    }

    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                decoded.push(hex);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(decoded)
        .map(Cow::Owned)
        .unwrap_or_else(|_| Cow::Borrowed(value))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::fs::document::MarkdownSource;

    use super::render_document;

    #[test]
    fn renders_toc_and_unique_heading_ids() {
        let source = MarkdownSource {
            path: PathBuf::from("note.md"),
            base_dir: PathBuf::from("."),
            content: "# Title\n\n## Repeat\n\n## Repeat\n\n```rust\nfn main() {}\n```\n"
                .to_string(),
        };

        let rendered = render_document(&source);

        assert_eq!(rendered.toc.len(), 3);
        assert_eq!(rendered.toc[1].id, "repeat");
        assert_eq!(rendered.toc[2].id, "repeat-2");
        assert!(rendered.html.contains("id=\"repeat-2\""));
        assert!(rendered.html.contains("<pre"));
    }

    #[test]
    fn renders_latex_math_events() {
        let source = MarkdownSource {
            path: PathBuf::from("math.md"),
            base_dir: PathBuf::from("."),
            content: "Inline $\\alpha^2$.\n\n$$\\frac{1}{2}$$\n".to_string(),
        };

        let rendered = render_document(&source);

        assert!(rendered.html.contains("math-inline"));
        assert!(rendered.html.contains("math-display"));
        assert!(rendered.html.contains("mfrac"));
        assert!(rendered.html.contains("α"));
    }
}
