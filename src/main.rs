use std::io;
mod parse_input;
use parse_input::*;
mod app;
mod command;
mod config;
mod execute;
mod render;

use crate::{
    app::State,
    command::{BuiltIn, Command, Runner},
    config::ShellConfig,
    render::render_prompt,
};

fn main() {
    let mut state = State::new();
    let config = ShellConfig::new("./shell.toml").unwrap();

    loop {
        state.refresh();
        render_prompt(&state, &config);

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let tokens = tokenize(input.trim());
        if tokens.is_empty() {
            continue;
        }

        let exec = &tokens[0];
        let args = &tokens[1..];

        let internal_command = Command::new(&exec, args);

        if let Some(command) = internal_command {
            if let Runner::InBuilt(com) = &command.runner {
                if com == &BuiltIn::Exit {
                    print!("\x1b[0m");
                    break;
                }
            }

            let return_code = command.exec(&mut state);
            // println!("{:?}", return_code);
        } else {
            println!("could not find \"{}\"", &exec);
        }
    }
}
