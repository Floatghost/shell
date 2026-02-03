use std::io::{self, Write};
mod parse_input;
use parse_input::*;
mod execute;
mod app;
mod command;

use crate::{app::State, command::{BuiltIn, Command, Runner}};

fn main() {
    let mut state = State::new();
    
    loop {
        
        println!("{}", state.username);
        print!("{}> ", state.cwd.to_str().unwrap());
        
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let tokens = tokenize(input.trim());

        let internal_command = Command::new(&tokens[0], &tokens[1..]);

        if let Some(command) = internal_command {
            if let Runner::InBuilt(com) = &command.runner {
                if com == &BuiltIn::Exit {
                    break;
                }
            }
            
            let return_code = command.exec(&mut state);
            println!("{:?}", return_code);
        }
    }
}
