use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::slice::GetDisjointMutError;

use serde::Deserialize;

pub const BUILTIN_SEGMENTS: &[&str] = &[
    "prompt.time",
    "prompt.date",
    "prompt.cwd",
    "prompt.git",
    "prompt.cpu",
    "prompt.ram",
    "prompt.username",
    "prompt.hostname",
];

#[derive(Debug)]
pub struct ShellConfig {
    pub theme: ThemeConfig,
    pub prompt: PromptConfig,
    pub segments: HashMap<String, ResolvedSegment>,
}

const DEFAULT_CONF: &str = include_str!("../shell.toml");

impl ShellConfig {
    pub fn new() -> Result<ShellConfig, Box<dyn std::error::Error>> {
        let filepath = ShellConfig::config_path();
        let content = match fs::read_to_string(&filepath) {
            Ok(c) => c,
            Err(e) => ShellConfig::generate_default_conf(&filepath)?,
        };
        let raw: RawShellConfig = toml::from_str(&content)?;
        Ok(raw.resolve_config())
    }

    fn generate_default_conf(filepath: &PathBuf) -> std::io::Result<String> {
        let contents = DEFAULT_CONF.to_string();

        if let Some(parent) = filepath.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(&filepath)?;
        file.write_all(contents.as_bytes())?;

        Ok(contents)
    }
    pub fn config_path() -> std::path::PathBuf {
        let appdata = std::env::var("APPDATA").expect("%APPDATA% not set (this is Windows-only)");

        std::path::PathBuf::from(appdata)
            .join("LeVimShell")
            .join("shell.toml")
    }
}

#[derive(Deserialize)]
pub struct RawShellConfig {
    pub theme: Option<ThemeConfig>,
    pub prompt: RawPromptConfig,
}

impl RawShellConfig {
    pub fn resolve_config(self) -> ShellConfig {
        let theme = self.theme.unwrap_or_default();
        let mut segments = HashMap::new();

        let user_flat_segments = self.prompt.to_flat_segments();

        for &name in BUILTIN_SEGMENTS {
            let defaults = default_segment_for(name);
            if let Some(user_seg) = user_flat_segments.get(name) {
                segments.insert(
                    name.to_string(),
                    ResolvedSegment {
                        enabled: user_seg.enabled.unwrap_or(defaults.enabled),
                        fg: user_seg.fg.clone().unwrap_or(defaults.fg),
                        bg: user_seg.bg.clone().unwrap_or(defaults.bg),
                        format: user_seg.format.clone().unwrap_or(defaults.format),
                    },
                );
            } else {
                segments.insert(name.to_string(), defaults);
            }
        }

        for (name, user_seg) in &user_flat_segments {
            if !BUILTIN_SEGMENTS.contains(&name.as_str()) {
                let defaults = default_segment_for(name);
                segments.insert(
                    name.clone(),
                    ResolvedSegment {
                        enabled: user_seg.enabled.unwrap_or(defaults.enabled),
                        fg: user_seg.fg.clone().unwrap_or(defaults.fg),
                        bg: user_seg.bg.clone().unwrap_or(defaults.bg),
                        format: user_seg.format.clone().unwrap_or(defaults.format),
                    },
                );
            }
        }

        ShellConfig {
            theme,
            prompt: PromptConfig {
                lines: self.prompt.lines,
            },
            segments,
        }
    }
}

#[derive(Deserialize)]
pub struct RawPromptConfig {
    pub lines: Vec<String>,

    #[serde(flatten)]
    pub nested_segments: HashMap<String, toml::Value>,
}

impl RawPromptConfig {
    pub fn to_flat_segments(&self) -> HashMap<String, PromptSegmentConfig> {
        let mut map = HashMap::new();
        for (key, val) in &self.nested_segments {
            let flat_key = format!("prompt.{}", key);

            if let Ok(seg) = val.clone().try_into::<PromptSegmentConfig>() {
                map.insert(flat_key, seg);
            }
        }
        map
    }
}

#[derive(Debug)]
pub struct PromptConfig {
    pub lines: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct PromptSegmentConfig {
    pub enabled: Option<bool>,
    pub fg: Option<ColorValue>,
    pub bg: Option<ColorValue>,
    pub format: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSegment {
    pub enabled: bool,
    pub fg: ColorValue,
    pub bg: ColorValue,
    pub format: String,
}

pub fn default_segment_for(name: &str) -> ResolvedSegment {
    match name {
        "prompt.time" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::White),
            bg: ColorValue::Named(Color::Default),
            format: "{HH}:{MM}:{SS}".to_string(),
        },
        "prompt.date" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::White),
            bg: ColorValue::Named(Color::Default),
            format: "{YYYY}-{MM}-{DD}".to_string(),
        },
        "prompt.cwd" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Cyan),
            bg: ColorValue::Named(Color::Default),
            format: "{cwd}".to_string(),
        },
        "prompt.cpu" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::White),
            bg: ColorValue::Named(Color::Default),
            format: "{cpu}%".to_string(),
        },
        "prompt.ram" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::White),
            bg: ColorValue::Named(Color::Default),
            format: "{ram}%".to_string(),
        },
        "prompt.git" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Magenta),
            bg: ColorValue::Named(Color::Default),
            format: "{branch}{dirty}".to_string(),
        },
        "prompt.username" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Green),
            bg: ColorValue::Named(Color::Default),
            format: "{username}".to_string(),
        },
        "prompt.hostname" => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Blue),
            bg: ColorValue::Named(Color::Default),
            format: "{hostname}".to_string(),
        },
        _ => ResolvedSegment {
            enabled: true,
            fg: ColorValue::Named(Color::Default),
            bg: ColorValue::Named(Color::Default),
            format: "{value}".to_string(),
        },
    }
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ColorValue {
    Named(Color),
    Hex(String),
}

impl ColorValue {
    pub fn to_ansi_fg(&self) -> String {
        match self {
            ColorValue::Named(c) => c.to_ansi_fg().to_string(),
            ColorValue::Hex(h) => {
                let (r, g, b) = parse_hex(h);
                format!("\x1b[38;2;{};{};{}m", r, g, b)
            }
        }
    }

    pub fn to_ansi_bg(&self) -> String {
        match self {
            ColorValue::Named(c) => c.to_ansi_bg().to_string(),
            ColorValue::Hex(h) => {
                let (r, g, b) = parse_hex(h);
                format!("\x1b[48;2;{};{};{}m", r, g, b)
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
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

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}
