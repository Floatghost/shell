mod parse_input;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
mod app;
mod command;
mod config;
mod execute;
mod find;
mod readline;
mod render;
mod userinput;

use crate::{
    app::State,
    command::{BuiltIn, Command, Runner},
    config::ShellConfig,
    parse_input::tokenize,
    readline::ReadLine,
    render::render_prompt,
};

fn main() {
    let mut state = State::new();
    let config = ShellConfig::new().unwrap();

    enable_raw_mode().unwrap();
    let _ = ExitStrat;

    let mut return_code = None;

    loop {
        state.refresh();
        let prompt = render_prompt(&state, &config);

        let last_prompt_line = prompt.lines().last().unwrap();

        let mut readline = ReadLine::new(last_prompt_line);
        let userinput = match readline.get_line() {
            Ok(n) => n,
            Err(e) => match e.as_str() {
                "ctrl-c" => return,
                _ => panic!("ERROR: {}", e),
            },
        };

        let tokens = tokenize(&userinput);

        let exec = &tokens[0];
        let args = &tokens[1..];

        let internal_command = Command::new(exec, args);

        if let Some(command) = internal_command {
            if let Runner::InBuilt(com) = &command.runner
                && com == &BuiltIn::Exit
            {
                print!("\x1b[0m");
                println!("return {:?}", return_code);
                break;
            }

            return_code = command.exec(&mut state);
        } else {
            println!("could not find \"{}\"", &exec);
        }
    }
}

struct ExitStrat;

impl Drop for ExitStrat {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
