/*
use std::collections::HashMap;
use std::io::Write;

use chrono::Local;
use sysinfo::System;

use crate::{
    app::State,
    // config::{ColorValue, ShellConfig},
};

pub fn render_prompt(state: &State, cfg: &ShellConfig) {
    let prompt = construct_prompt(state, cfg);

    print!("{}", prompt);

    print!("\x1b[0m");

    std::io::stdout().flush().unwrap();
}

fn construct_prompt(state: &State, cfg: &ShellConfig) -> String {
    let mut blocks = Blocks::new();

    let def_fg = cfg.theme.fg.clone();
    let def_bg = cfg.theme.bg.clone();

    for (idx, line) in cfg.prompt.lines.iter().enumerate() {
        let tokens = tokenize(line);
        for token in tokens {
            match token {
                Token::Text(text) => blocks.push(&text, def_fg.clone(), def_bg.clone()),
                Token::Placeholder(key) => {
                    if let Some((text, (fg, bg))) = render_segment_by_name(&key, state, cfg) {
                        blocks.push(&text, fg, bg);
                    } else {
                    }
                }
            }
        }
        if idx != cfg.prompt.lines.len() - 1 {
            blocks.push("\n", def_fg.clone(), def_bg.clone());
        }
    }

    blocks.build()
}

#[derive(Debug, Clone)]
pub enum Token {
    Text(String),
    Placeholder(String),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_text = String::new();
    let mut inside_placeholder = false;

    for c in input.chars() {
        if c == '{' {
            if !inside_placeholder {
                if !current_text.is_empty() {
                    tokens.push(Token::Text(current_text.clone()));
                    current_text.clear();
                }
                inside_placeholder = true;
            } else {
                // Nested '{' or "{{" - treat previous as text part of placeholder?
                // For simplicity, just append to current key
                current_text.push(c);
            }
        } else if c == '}' {
            if inside_placeholder {
                tokens.push(Token::Placeholder(current_text.clone()));
                current_text.clear();
                inside_placeholder = false;
            } else {
                current_text.push(c);
            }
        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        if inside_placeholder {
            tokens.push(Token::Text(format!("{{{}", current_text)));
        } else {
            tokens.push(Token::Text(current_text));
        }
    }

    tokens
}

fn render_segment_generic<F>(
    segment_name: &str,
    state: &State,
    cfg: &ShellConfig,
    build_map: F,
) -> Option<(String, (ColorValue, ColorValue))>
where
    F: Fn(&State) -> Option<HashMap<String, String>>,
{
    let seg_cfg = cfg.segments.get(segment_name)?;

    if !seg_cfg.enabled {
        return None;
    }

    let values = build_map(state)?;

    let tokens = tokenize(&seg_cfg.format);
    let mut output = String::new();

    for token in tokens {
        match token {
            Token::Text(t) => output.push_str(&t),
            Token::Placeholder(key) => {
                if let Some(val) = values.get(&key) {
                    output.push_str(val);
                }
            }
        }
    }

    Some((output, (seg_cfg.fg.clone(), seg_cfg.bg.clone())))
}

fn render_segment_by_name(
    name: &str,
    state: &State,
    cfg: &ShellConfig,
) -> Option<(String, (ColorValue, ColorValue))> {
    let full_key = if name.starts_with("prompt.") {
        name.to_string()
    } else {
        format!("prompt.{}", name)
    };

    match name {
        "cwd" | "prompt.cwd" => render_segment_generic(&full_key, state, cfg, |s| {
            let mut map = HashMap::new();
            map.insert("cwd".to_string(), s.cwd.display().to_string());
            Some(map)
        }),
        "username" | "prompt.username" => render_segment_generic(&full_key, state, cfg, |s| {
            let mut map = HashMap::new();
            map.insert("username".to_string(), s.username.clone());
            Some(map)
        }),
        "time" | "prompt.time" => render_segment_generic(&full_key, state, cfg, |_| {
            let now = Local::now();
            let mut map = HashMap::new();
            map.insert("HH".to_string(), now.format("%H").to_string());
            map.insert("MM".to_string(), now.format("%M").to_string());
            map.insert("SS".to_string(), now.format("%S").to_string());
            map.insert("time".to_string(), now.format("%H:%M:%S").to_string());
            Some(map)
        }),
        "date" | "prompt.date" => render_segment_generic(&full_key, state, cfg, |_| {
            let now = Local::now();
            let mut map = HashMap::new();
            map.insert("YYYY".to_string(), now.format("%Y").to_string());
            map.insert("MM".to_string(), now.format("%m").to_string());
            map.insert("DD".to_string(), now.format("%d").to_string());
            Some(map)
        }),
        "git" | "prompt.git" => render_segment_generic(&full_key, state, cfg, |s| {
            let git = s.git.as_ref()?;
            let mut map = HashMap::new();
            map.insert("branch".to_string(), git.branch.clone());
            map.insert(
                "dirty".to_string(),
                if git.dirty {
                    "*".to_string()
                } else {
                    "".to_string()
                },
            );
            Some(map)
        }),
        "ram" | "prompt.ram" => render_segment_generic(&full_key, state, cfg, |s| {
            let used = s.system.used_memory();
            let total = s.system.total_memory();
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let mut map = HashMap::new();
            map.insert(
                "used".to_string(),
                format!("{:.2}GB", used as f64 / 1024.0 / 1024.0 / 1000.0),
            );
            map.insert(
                "total".to_string(),
                format!("{:.2}GB", total as f64 / 1024.0 / 1024.0 / 1000.0),
            );
            map.insert("percent".to_string(), format!("{:.0}", percent));
            map.insert("ram".to_string(), format!("{:.0}", percent));
            Some(map)
        }),
        "cpu" | "prompt.cpu" => render_segment_generic(&full_key, state, cfg, |s| {
            let usage = s.system.global_cpu_usage();
            let mut map = HashMap::new();
            map.insert("usage".to_string(), format!("{:.0}", usage));
            map.insert("cpu".to_string(), format!("{:.0}", usage));
            Some(map)
        }),
        "hostname" | "prompt.hostname" => render_segment_generic(&full_key, state, cfg, |_| {
            let host = System::host_name().unwrap_or_else(|| "localhost".to_string());
            let mut map = HashMap::new();
            map.insert("hostname".to_string(), host);
            Some(map)
        }),
        _ => None, // or render Unknown
    }
}

pub struct Blocks {
    pub data: Vec<(String, (ColorValue /*fg*/, ColorValue /*bg*/))>,
}

impl Blocks {
    pub fn new() -> Blocks {
        Blocks { data: Vec::new() }
    }

    pub fn push(&mut self, data: &str, fg: ColorValue, bg: ColorValue) {
        self.data.push((data.to_string(), (fg, bg)));
    }

    pub fn build(self) -> String {
        let mut out = String::new();

        for (value, (fg, bg)) in self.data {
            out.push_str(&format!(
                "{}{}{}\x1b[0m",
                fg.to_ansi_fg(),
                bg.to_ansi_bg(),
                value
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = "Hello {name}!";
        let tokens = tokenize(input);

        assert_eq!(tokens.len(), 3);
        match &tokens[0] {
            Token::Text(t) => assert_eq!(t, "Hello "),
            _ => panic!("Expected text"),
        }
        match &tokens[1] {
            Token::Placeholder(p) => assert_eq!(p, "name"),
            _ => panic!("Expected placeholder"),
        }
        match &tokens[2] {
            Token::Text(t) => assert_eq!(t, "!"),
            _ => panic!("Expected text"),
        }
    }

    #[test]

    fn test_tokenize_nested() {
        let input = "{a}{b}";

        let tokens = tokenize(input);

        assert_eq!(tokens.len(), 2);

        match &tokens[0] {
            Token::Placeholder(p) => assert_eq!(p, "a"),

            _ => panic!("Expected placeholder a"),
        }
    }
}
*/
