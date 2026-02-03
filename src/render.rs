use std::io::Write;

use crate::{app::State, config::ShellConfig};

pub fn render_prompt(state: &State, cfg: &ShellConfig) {
    let prompt = construct_prompt(state, cfg);

    print!("{}", prompt);

    print!("\x1b[0m");

    std::io::stdout().flush().unwrap();
}

fn construct_prompt(state: &State, cfg: &ShellConfig) -> String {
    let mut out = String::new();

    let tokens = tokenize_prompt(&cfg.prompt.lines);

    for token in tokens {
        match token {
            Token::Text(text) => out.push_str(&text),
            Token::NewLine => out.push('\n'),
            Token::Value(val) => out.push_str(&val.get_value(state)),
        }
    }

    out
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

    pub fn get_value(&self, state: &State) -> String {
        match self {
            Insert::CWD => Insert::handle_cwd(state),
            Insert::Username => Insert::handle_username(state),
            Insert::Hostname => Insert::handle_hostname(state),
            Insert::Git => Insert::handle_git(state),
            Insert::Ram => Insert::handle_ram(state),
            Insert::Cpu => Insert::handle_cpu(state),
            Insert::Time => Insert::handle_time(state),
            Insert::Date => Insert::handle_date(state),
            Insert::Unknown(unknown) => format!("(UNKNOWN INSERT: {})", unknown),
        }
    }

    fn handle_cwd(state: &State) -> String {
        // todo do it the proper way
        state.cwd.to_str().unwrap().to_string()
    }
    fn handle_username(state: &State) -> String {
        // todo do it the proper way
        state.username.clone()
    }
    fn handle_hostname(state: &State) -> String {
        todo!()
    }
    fn handle_git(state: &State) -> String {
        todo!()
    }
    fn handle_ram(state: &State) -> String {
        todo!()
    }
    fn handle_cpu(state: &State) -> String {
        todo!()
    }
    fn handle_time(state: &State) -> String {
        todo!()
    }
    fn handle_date(state: &State) -> String {
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
