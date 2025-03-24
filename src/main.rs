use std::io::{self, Write};
mod parse_input;
use parse_input::*;
mod execute;
use execute::*;
use std::env;

fn main() {
    
    loop {
        //get User input
        match env::var("USERNAME") {
            Ok(name) => {
                print!("{} $ ", name);
            }
            Err(_) => print!("$ "),
        }
        //print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        dbg!(&input);
        
        //triming and tokenize
        input = input.trim().to_string();
        let tokens = tokenize(&input);
        //dbg!(&tokens);
        
        //Deviding up in sections using ( &, |, > ) so multiple commands can be executed at a time
        let commands = group_commands(&tokens);

        //loop execute command and potentialy pass output to next input
        for command in commands {
            if let Some(return_code) = execute_command(command) {
                println!("{}", return_code);
                break;
            }
        }

        //if command is exit command exit

        break;
    }
}
