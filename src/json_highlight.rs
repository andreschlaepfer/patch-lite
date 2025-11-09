use iced::widget::text::{Rich, Span};
use iced::{Color, Font};
use serde_json::Value;

/// Tema de cores (estilo "Postman-ish").
#[derive(Clone, Copy)]
pub struct Theme {
    pub key: Color,
    pub string: Color,
    pub number: Color,
    pub boolean: Color,
    pub null_: Color,
    pub punct: Color,
    pub default: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            key: Color::from_rgb8(67, 156, 255),
            string: Color::from_rgb8(80, 250, 123),
            number: Color::from_rgb8(255, 184, 108),
            boolean: Color::from_rgb8(189, 147, 249),
            null_: Color::from_rgb8(139, 139, 139),
            punct: Color::from_rgb8(120, 120, 120),
            default: Color::from_rgb8(220, 220, 220),
        }
    }
}

/// Converte um `&str` contendo JSON em `Rich<'static, ()>`.
/// Se o JSON for inválido, mostra um aviso + conteúdo original sem highlight.
pub fn rich_json_str(src: &str) -> Rich<'static, ()> {
    match serde_json::from_str::<Value>(src) {
        Ok(v) => rich_json_value(&v),
        Err(e) => {
            let mut spans = Vec::new();
            spans.push(
                Span::new(format!("❌ JSON inválido: {e}\n\n"))
                    .color(Color::from_rgb8(255, 100, 100)),
            );
            spans.push(Span::new(src.to_owned()).color(Theme::default().default));
            Rich::with_spans(spans).font(Font::MONOSPACE).size(14)
        }
    }
}

/// Versão para `serde_json::Value`.
pub fn rich_json_value(value: &Value) -> Rich<'static, ()> {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| "<invalid json>".into());
    rich_json_pretty_str(&pretty, Theme::default())
}

/// Mesmo que `rich_json_str`, mas recebendo:
/// - o JSON já "pretty" (com quebras e indentação)
/// - um tema customizável
pub fn rich_json_pretty_str(pretty_src: &str, theme: Theme) -> Rich<'static, ()> {
    let spans = json_to_spans(pretty_src, theme);
    Rich::with_spans(spans).font(Font::MONOSPACE).size(14)
}

/// Útil para logs/clipboard: apenas identa (sem cores).
pub fn pretty_json_str(src: &str) -> String {
    match serde_json::from_str::<Value>(src) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| src.to_string()),
        Err(_) => src.to_string(),
    }
}

fn json_to_spans(src: &str, th: Theme) -> Vec<Span<'static>> {
    #[derive(Clone, Copy)]
    enum Kind {
        Default,
        Key,
        String,
        Number,
        Bool,
        Null,
        Punct,
    }

    let mut out: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();

    let chars: Vec<char> = src.chars().collect();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escape = false;

    let flush = |k: Kind, b: &mut String, out: &mut Vec<Span<'static>>| {
        if b.is_empty() {
            return;
        }
        let color = match k {
            Kind::Key => th.key,
            Kind::String => th.string,
            Kind::Number => th.number,
            Kind::Bool => th.boolean,
            Kind::Null => th.null_,
            Kind::Punct => th.punct,
            Kind::Default => th.default,
        };
        out.push(Span::new(std::mem::take(b)).color(color));
    };

    while i < chars.len() {
        let c = chars[i];

        if in_string {
            buf.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                // Fechou string -> decidir se é "Key" olhando próximo token significativo
                let mut kind = Kind::String;
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && chars[j] == ':' {
                    kind = Kind::Key;
                }
                flush(kind, &mut buf, &mut out);
                in_string = false;
            }
            i += 1;
            continue;
        }

        match c {
            '"' => {
                flush(Kind::Default, &mut buf, &mut out);
                in_string = true;
                buf.push(c);
                i += 1;
            }
            ':' | '{' | '}' | '[' | ']' | ',' => {
                flush(Kind::Default, &mut buf, &mut out);
                out.push(Span::new(c.to_string()).color(th.punct));
                i += 1;
            }
            _ if c.is_ascii_digit() || c == '-' => {
                flush(Kind::Default, &mut buf, &mut out);
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || ".eE+-".contains(chars[i])) {
                    i += 1;
                }
                let num: String = chars[start..i].iter().collect();
                out.push(Span::new(num).color(th.number));
            }
            't' if src[i..].starts_with("true") => {
                flush(Kind::Default, &mut buf, &mut out);
                out.push(Span::new("true").color(th.boolean));
                i += 4;
            }
            'f' if src[i..].starts_with("false") => {
                flush(Kind::Default, &mut buf, &mut out);
                out.push(Span::new("false").color(th.boolean));
                i += 5;
            }
            'n' if src[i..].starts_with("null") => {
                flush(Kind::Default, &mut buf, &mut out);
                out.push(Span::new("null").color(th.null_));
                i += 4;
            }
            _ => {
                buf.push(c);
                i += 1;
            }
        }
    }

    flush(Kind::Default, &mut buf, &mut out);
    out
}
