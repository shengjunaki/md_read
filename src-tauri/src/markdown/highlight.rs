use html_escape::encode_safe;
use std::borrow::Cow;
use std::str::FromStr;
use syntect::{
    highlighting::{Color, FontStyle, StyleModifier, Theme, ThemeItem, ThemeSettings},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

pub struct Highlighter {
    syntaxes: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new() -> Self {
        let syntaxes = SyntaxSet::load_defaults_newlines();
        let theme = mocha_theme();
        Self { syntaxes, theme }
    }

    pub fn highlight(&self, language: &str, code: &str) -> String {
        let language = normalize_language(language);
        let syntax = self
            .syntaxes
            .find_syntax_by_token(&language)
            .or_else(|| self.syntaxes.find_syntax_by_extension(&language))
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text());

        highlighted_html_for_string(code, &self.syntaxes, syntax, &self.theme)
            .unwrap_or_else(|_| plain_code(&language, code))
    }
}

fn mocha_theme() -> Theme {
    Theme {
        name: Some("Catppuccin Mocha Reader".to_string()),
        author: Some("Markdown Reader".to_string()),
        settings: ThemeSettings {
            foreground: Some(color(0xcdd6f4)),
            background: Some(color(0x181825)),
            accent: Some(color(0x89b4fa)),
            selection: Some(color(0x45475a)),
            ..ThemeSettings::default()
        },
        scopes: vec![
            item("comment", 0x6c7086, Some(FontStyle::ITALIC)),
            item("string", 0xa6e3a1, None),
            item("constant.numeric", 0xfab387, None),
            item("constant.language", 0xfab387, None),
            item("constant.character", 0xfab387, None),
            item("keyword", 0xcba6f7, None),
            item("storage", 0xcba6f7, None),
            item("operator", 0x89dceb, None),
            item("entity.name.function", 0x89b4fa, None),
            item("support.function", 0x89b4fa, None),
            item("variable.function", 0x89b4fa, None),
            item("entity.name.type", 0xf9e2af, None),
            item("support.type", 0xf9e2af, None),
            item("variable", 0xcdd6f4, None),
            item("variable.parameter", 0xf2cdcd, None),
            item("punctuation", 0x9399b2, None),
            item("invalid", 0xf38ba8, Some(FontStyle::UNDERLINE)),
        ],
    }
}

fn item(scope: &str, foreground: u32, font_style: Option<FontStyle>) -> ThemeItem {
    ThemeItem {
        scope: syntect::highlighting::ScopeSelectors::from_str(scope)
            .expect("hardcoded syntect scope selector should parse"),
        style: StyleModifier {
            foreground: Some(color(foreground)),
            background: None,
            font_style,
        },
    }
}

fn color(rgb: u32) -> Color {
    Color {
        r: ((rgb >> 16) & 0xff) as u8,
        g: ((rgb >> 8) & 0xff) as u8,
        b: (rgb & 0xff) as u8,
        a: 0xff,
    }
}

fn normalize_language(language: &str) -> Cow<'_, str> {
    match language.to_ascii_lowercase().as_str() {
        "js" | "jsx" => Cow::Borrowed("javascript"),
        "ts" | "tsx" => Cow::Borrowed("typescript"),
        "sh" | "shell" | "zsh" => Cow::Borrowed("bash"),
        "ps" | "ps1" => Cow::Borrowed("powershell"),
        "py" => Cow::Borrowed("python"),
        "rs" => Cow::Borrowed("rust"),
        "yml" => Cow::Borrowed("yaml"),
        "md" => Cow::Borrowed("markdown"),
        "jsonc" => Cow::Borrowed("json"),
        _ => Cow::Borrowed(language),
    }
}

pub fn plain_code(language: &str, code: &str) -> String {
    format!(
        "<pre><code class=\"language-{}\">{}</code></pre>",
        encode_safe(language),
        encode_safe(code)
    )
}
