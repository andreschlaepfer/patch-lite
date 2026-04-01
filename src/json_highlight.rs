use iced::advanced::text::highlighter::{self, Highlighter};
use iced::{Color, Font};
use serde_json::Value;
use std::ops::Range;

/// Tema de cores (estilo "Postman-ish").
#[derive(Clone, Copy)]
pub struct HighlightTheme {
    pub key: Color,
    pub string: Color,
    pub number: Color,
    pub boolean: Color,
    pub null_: Color,
    pub punct: Color,
}

impl Default for HighlightTheme {
    fn default() -> Self {
        Self {
            key: Color::from_rgb8(67, 156, 255),
            string: Color::from_rgb8(80, 250, 123),
            number: Color::from_rgb8(255, 184, 108),
            boolean: Color::from_rgb8(189, 147, 249),
            null_: Color::from_rgb8(139, 139, 139),
            punct: Color::from_rgb8(120, 120, 120),
        }
    }
}

/// Pretty-prints JSON (without colors).
pub fn pretty_json_str(src: &str) -> String {
    match serde_json::from_str::<Value>(src) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| src.to_string()),
        Err(_) => src.to_string(),
    }
}

// --- text_editor Highlighter for JSON syntax highlighting ---

#[derive(Debug, Clone, PartialEq)]
pub struct JsonHighlighterSettings;

pub struct JsonHighlighter {
    current_line: usize,
}

#[derive(Debug, Clone)]
pub struct JsonHighlight(Color);

impl JsonHighlight {
    pub fn to_format(&self) -> highlighter::Format<Font> {
        highlighter::Format {
            color: Some(self.0),
            font: None,
        }
    }
}

impl Highlighter for JsonHighlighter {
    type Settings = JsonHighlighterSettings;
    type Highlight = JsonHighlight;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = self.current_line.min(line);
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        self.current_line += 1;

        let th = HighlightTheme::default();
        let mut result: Vec<(Range<usize>, JsonHighlight)> = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0usize;
        let mut in_string = false;
        let mut escape = false;
        let mut string_start = 0usize;

        while i < chars.len() {
            let byte_pos = chars[..i].iter().map(|c| c.len_utf8()).sum::<usize>();
            let c = chars[i];

            if in_string {
                if escape {
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    let end_byte = chars[..=i].iter().map(|c| c.len_utf8()).sum::<usize>();
                    // Determine if this is a key (followed by ':')
                    let mut j = i + 1;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    let color = if j < chars.len() && chars[j] == ':' {
                        th.key
                    } else {
                        th.string
                    };
                    result.push((string_start..end_byte, JsonHighlight(color)));
                    in_string = false;
                }
                i += 1;
                continue;
            }

            match c {
                '"' => {
                    string_start = byte_pos;
                    in_string = true;
                    i += 1;
                }
                ':' | '{' | '}' | '[' | ']' | ',' => {
                    let end_byte = byte_pos + c.len_utf8();
                    result.push((byte_pos..end_byte, JsonHighlight(th.punct)));
                    i += 1;
                }
                _ if c.is_ascii_digit() || c == '-' => {
                    let start_byte = byte_pos;
                    i += 1;
                    while i < chars.len()
                        && (chars[i].is_ascii_digit() || ".eE+-".contains(chars[i]))
                    {
                        i += 1;
                    }
                    let end_byte = chars[..i].iter().map(|c| c.len_utf8()).sum::<usize>();
                    result.push((start_byte..end_byte, JsonHighlight(th.number)));
                }
                't' if line[byte_pos..].starts_with("true") => {
                    result.push((byte_pos..byte_pos + 4, JsonHighlight(th.boolean)));
                    i += 4;
                }
                'f' if line[byte_pos..].starts_with("false") => {
                    result.push((byte_pos..byte_pos + 5, JsonHighlight(th.boolean)));
                    i += 5;
                }
                'n' if line[byte_pos..].starts_with("null") => {
                    result.push((byte_pos..byte_pos + 4, JsonHighlight(th.null_)));
                    i += 4;
                }
                _ => {
                    i += 1;
                }
            }
        }

        result.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}
