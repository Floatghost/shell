use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ShellConfig {
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    pub fg: Color,
    pub bg: Color,
}

impl ShellConfig {
    pub fn new(filepath: &str) -> Result<ShellConfig, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(filepath)?;
        let config: ShellConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
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