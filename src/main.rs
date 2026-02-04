use std::io;
mod parse_input;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use parse_input::*;
mod app;
mod command;
mod config;
mod execute;
mod render;
mod userinput;

use crate::{
    app::State,
    command::{BuiltIn, Command, Runner},
    config::ShellConfig, userinput::get_userinput,
    render::render_prompt,
};

fn main() {
    let mut state = State::new();
    let config = ShellConfig::new("./shell.toml").unwrap();

    let exit = ExitStrat;

    enable_raw_mode().unwrap();

    userinput::get_userinput(&state, &config);


    loop {
        state.refresh();
        render_prompt(&state, &config);

        let input = match get_userinput(&state, &config) {
            Some(n) => n,
            None => continue,
        };

        if input.is_empty() {
            continue;
        }

        /*
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
        */
    }
}

struct ExitStrat;

impl Drop for ExitStrat {
    fn drop(&mut self) {
        disable_raw_mode();
    }
}
