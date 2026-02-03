use std::io::Write;

use crate::{
    app::State,
    config::{Color, ColorValue, ShellConfig},
};

pub fn render_prompt(state: &State, cfg: &ShellConfig) {
    let prompt = construct_prompt(state, cfg);

    print!("{}", prompt);

    print!("\x1b[0m");

    std::io::stdout().flush().unwrap();
}

fn construct_prompt(state: &State, cfg: &ShellConfig) -> String {
    let tokens = tokenize_prompt(&cfg.prompt.lines);

    let mut blocks = Blocks::new();

    let def_fg = cfg.theme.fg.clone();
    let def_bg = cfg.theme.bg.clone();

    for token in tokens {
        match token {
            Token::Text(text) => blocks.push(&text, def_fg.clone(), def_bg.clone()),
            Token::NewLine => blocks.push("\n", def_fg.clone(), def_bg.clone()),
            Token::Value(val) => {
                let (data, (fg, bg)) = val.get_value(state, cfg);
                blocks.push(&data, fg, bg);
            }
        }
    }

    blocks.build()
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

pub enum Token {
    Text(String),
    NewLine,
    Value(Insert),
}

pub enum Insert {
    CWD,
    Username,
    Hostname,
    Git,
    Ram,
    Cpu,
    Time,
    Date,
    Unknown(String),
}

impl Insert {
    pub fn from(input: String) -> Insert {
        match input.as_str() {
            "cwd" => Insert::CWD,
            "username" => Insert::Username,
            "hostname" => Insert::Hostname,
            "git" => Insert::Git,
            "ram" => Insert::Ram,
            "cpu" => Insert::Cpu,
            "time" => Insert::Time,
            "date" => Insert::Date,
            _ => Insert::Unknown(input.clone()),
        }
    }

    pub fn get_value(
        &self,
        state: &State,
        cfg: &ShellConfig,
    ) -> (String, (ColorValue, ColorValue)) {
        match self {
            Insert::CWD => Insert::handle_cwd(state, cfg),
            Insert::Username => Insert::handle_username(state, cfg),
            Insert::Hostname => Insert::handle_hostname(state, cfg),
            Insert::Git => Insert::handle_git(state, cfg),
            Insert::Ram => Insert::handle_ram(state, cfg),
            Insert::Cpu => Insert::handle_cpu(state, cfg),
            Insert::Time => Insert::handle_time(state, cfg),
            Insert::Date => Insert::handle_date(state, cfg),
            Insert::Unknown(unknown) => Some((
                format!("(UNKNOWN INSERT: {})", unknown),
                (
                    ColorValue::Named(Color::Red),
                    ColorValue::Named(Color::Default),
                ),
            )),
        }
        .unwrap_or((
            "".to_string(),
            (
                ColorValue::Named(Color::White),
                ColorValue::Named(Color::Default),
            ),
        ))
    }

    fn handle_cwd(state: &State, cfg: &ShellConfig) -> Option<(String, (ColorValue, ColorValue))> {
        let config = cfg.segments.get("prompt.cwd").unwrap();

        if !config.enabled {
            return None;
        }

        let fg = config.fg.clone();
        let bg = config.bg.clone();

        // todo do it the proper way
        Some((state.cwd.to_str().unwrap().to_string(), (fg, bg)))
    }
    fn handle_username(
        state: &State,
        cfg: &ShellConfig,
    ) -> Option<(String, (ColorValue, ColorValue))> {
        let config = cfg.segments.get("prompt.username").unwrap();

        if !config.enabled {
            return None;
        }

        let fg = config.fg.clone();
        let bg = config.bg.clone();

        let format_tokens = tokenize_prompt(&vec![config.format.clone()]);

        // todo do it the proper way
        Some((state.username.clone(), (fg, bg)))
    }
    fn handle_hostname(
        state: &State,
        cfg: &ShellConfig,
    ) -> Option<(String, (ColorValue, ColorValue))> {
        todo!()
    }
    fn handle_git(state: &State, cfg: &ShellConfig) -> Option<(String, (ColorValue, ColorValue))> {
        todo!()
    }
    fn handle_ram(state: &State, cfg: &ShellConfig) -> Option<(String, (ColorValue, ColorValue))> {
        todo!()
    }
    fn handle_cpu(state: &State, cfg: &ShellConfig) -> Option<(String, (ColorValue, ColorValue))> {
        todo!()
    }
    fn handle_time(state: &State, cfg: &ShellConfig) -> Option<(String, (ColorValue, ColorValue))> {
        todo!()
    }
    fn handle_date(state: &State, cfg: &ShellConfig) -> Option<(String, (ColorValue, ColorValue))> {
        todo!()
    }
}

fn tokenize_prompt(lines: &Vec<String>) -> Vec<Token> {
    let mut out = Vec::new();
    let mut temp = String::new();

    for (idx, line) in lines.iter().enumerate() {
        for c in line.chars() {
            match c {
                '{' => {
                    if !temp.is_empty() {
                        out.push(Token::Text(temp.clone()));
                        temp.clear();
                    }
                }
                '}' => {
                    if !temp.is_empty() {
                        out.push(Token::Value(Insert::from(temp.clone())));
                        temp.clear();
                    }
                }
                a => temp.push(a),
            }
        }

        if !temp.is_empty() {
            out.push(Token::Text(temp.clone()));
            temp.clear();
        }

        if idx != lines.len() - 1 {
            out.push(Token::NewLine);
        }
    }

    out
}
