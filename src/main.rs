use std::io::{self, Write};
mod parse_input;
use parse_input::*;
mod app;
mod command;
mod config;
mod execute;

use crate::{
    app::State,
    command::{BuiltIn, Command, Runner},
    config::ShellConfig,
};

fn main() {
    let mut state = State::new();
    let config = ShellConfig::new("./shell.toml").unwrap();

    dbg!(&config);
    
    loop {
        render_prompt(&state, &config);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let tokens = tokenize(input.trim());
        if tokens.is_empty() {
            continue;
        }

        let internal_command = Command::new(&tokens[0], &tokens[1..]);

        if let Some(command) = internal_command {
            if let Runner::InBuilt(com) = &command.runner {
                if com == &BuiltIn::Exit {
                    print!("\x1b[0m");
                    break;
                }
            }

            let return_code = command.exec(&mut state);
            println!("{:?}", return_code);
        }
    }
}

fn render_prompt(state: &State, cfg: &ShellConfig) {
    let fg = cfg.theme.fg.to_ansi_fg();
    let bg = cfg.theme.bg.to_ansi_bg();
    
    // farbige Zeile (z. B. Username)
    print!("{}{}{}", fg, bg, state.username);

    // CMD-Prompt
    println!("{}{} {}\x1b[0m", fg, bg, state.cwd.to_str().unwrap());
    print!("{}{}> \x1b[0m", fg, bg);
}
