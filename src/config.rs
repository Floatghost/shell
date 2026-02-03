use std::{collections::HashMap, fs};

use serde::Deserialize;

#[derive(Debug)]
pub struct ShellConfig {
    pub theme: ThemeConfig,

    pub prompt: PromptConfig,

    pub segments: HashMap<String, ResolvedSegment>,
}

impl ShellConfig {
    pub fn new(filepath: &str) -> Result<ShellConfig, Box<dyn std::error::Error>> {
        let config = RawShellConfig::new(filepath)?;
        
        dbg!(&config);
        
        Ok(config.resolve_config())
    }
}

#[derive(Debug, Deserialize)]
pub struct RawShellConfig {
    pub theme: Option<ThemeConfig>,

    pub prompt: PromptConfig,

    #[serde(flatten)]
    pub segments: HashMap<String, PromptSegmentConfig>,
}

pub const BUILTIN_SEGMENTS: &[&str] = &[
    "prompt.time",
    "prompt.date",
    "prompt.cwd",
    "prompt.git",
    "prompt.username",
    "prompt.hostname",
];

impl RawShellConfig {
    pub fn new(filepath: &str) -> Result<RawShellConfig, Box<dyn std::error::Error>> {
        let config: RawShellConfig = toml::from_str(&fs::read_to_string(filepath)?)?;

        Ok(config)
    }

    pub fn resolve_config(self) -> ShellConfig {
        let theme = self.theme.unwrap_or_default();

        let mut resolved = HashMap::new();

        for name in BUILTIN_SEGMENTS {
            let raw = self.segments.get(*name);
            let seg = match raw {
                Some(cfg) => cfg.resolve(name),
                None => default_segment_for(name),
            };
            resolved.insert(name.to_string(), seg);
        }

        for (name, raw) in self.segments.iter() {
            if !BUILTIN_SEGMENTS.contains(&name.as_str()) {
                resolved.insert(name.clone(), raw.resolve(name));
            }
        }

        ShellConfig {
            theme,
            prompt: self.prompt,
            segments: resolved,
        }
    }

    pub fn resolve_segment(&self, name: &str) -> ResolvedSegment {
        if let Some(raw) = self.segments.get(name) {
            raw.resolve(name)
        } else {
            default_segment_for(name)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    pub fg: ColorValue,
    pub bg: ColorValue,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            fg: ColorValue::Named(Color::Default),
            bg: ColorValue::Named(Color::Default),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PromptConfig {
    pub lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PromptSegmentConfig {
    pub enabled: Option<bool>,
    pub fg: Option<ColorValue>,
    pub bg: Option<ColorValue>,
    pub format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSegment {
    pub enabled: bool,
    pub fg: ColorValue,
    pub bg: ColorValue,
    pub format: String,
}

impl PromptSegmentConfig {
    pub fn resolve(&self, name: &str) -> ResolvedSegment {
        let defaults = default_segment_for(name);

        ResolvedSegment {
            enabled: self.enabled.unwrap_or(defaults.enabled),
            fg: self.fg.clone().unwrap_or(defaults.fg),
            bg: self.bg.clone().unwrap_or(defaults.bg),
            format: self.format.clone().unwrap_or(defaults.format),
        }
    }
}

pub fn default_segment_for(name: &str) -> ResolvedSegment {
    match name {
        "prompt.time" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::White),
            bg: ColorValue::Named(Color::Default),
            format: "{HH}:{MM}:{SS}".into(),
        },

        "prompt.date" => ResolvedSegment {
            enabled: false,
            fg: ColorValue::Named(Color::White),
            bg: ColorValue::Named(Color::Default),
            format: "{YYYY}-{MM}-{DD}".into(),
        },

        "prompt.cwd" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Cyan),
            bg: ColorValue::Named(Color::Default),
            format: "{cwd}".into(),
        },

        "prompt.git" => ResolvedSegment {
            enabled: false,
            fg: ColorValue::Named(Color::Magenta),
            bg: ColorValue::Named(Color::Default),
            format: "{branch}{dirty}".into(),
        },

        "prompt.username" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Green),
            bg: ColorValue::Named(Color::Default),
            format: "{username}".into(),
        },

        "prompt.hostname" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Blue),
            bg: ColorValue::Named(Color::Default),
            format: "{hostname}".into(),
        },

        _ => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Default),
            bg: ColorValue::Named(Color::Default),
            format: "{value}".into(),
        },
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ColorValue {
    Named(Color),
    Hex(String),
}

impl ColorValue {
    pub fn to_ansi_fg(&self) -> String {
        match self {
            ColorValue::Named(c) => c.to_ansi_fg().into(),
            ColorValue::Hex(h) => hex_to_ansi_fg(h),
        }
    }

    pub fn to_ansi_bg(&self) -> String {
        match self {
            ColorValue::Named(c) => c.to_ansi_bg().into(),
            ColorValue::Hex(h) => hex_to_ansi_bg(h),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

impl Color {
    pub fn to_ansi_fg(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::Default => "\x1b[39m",
        }
    }

    pub fn to_ansi_bg(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[40m",
            Color::Red => "\x1b[41m",
            Color::Green => "\x1b[42m",
            Color::Yellow => "\x1b[43m",
            Color::Blue => "\x1b[44m",
            Color::Magenta => "\x1b[45m",
            Color::Cyan => "\x1b[46m",
            Color::White => "\x1b[47m",
            Color::Default => "\x1b[49m",
        }
    }
}

fn hex_to_ansi_fg(hex: &str) -> String {
    let rgb = parse_hex_color(hex);
    format!("\x1b[38;2;{};{};{}m", rgb.0, rgb.1, rgb.2)
}

fn hex_to_ansi_bg(hex: &str) -> String {
    let rgb = parse_hex_color(hex);
    format!("\x1b[48;2;{};{};{}m", rgb.0, rgb.1, rgb.2)
}

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let clean = hex.trim_start_matches('#');
    if clean.len() != 6 {
        return (255, 255, 255);
    }

    let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(255);

    (r, g, b)
}
