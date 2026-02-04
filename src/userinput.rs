use std::{io::Write, path::PathBuf, time::Duration};

use crossterm::{
    cursor::position,
    event::{poll, read, Event, KeyCode, KeyEvent},
};

use crate::{app::State, config::ShellConfig};

pub enum UserInput {
    Path(PathBuf),
    Ident(String),
    Args(String),
}

pub fn get_userinput(state: &State, config: &ShellConfig) -> Option<Vec<UserInput>> {
    let mut input_buffer = String::new();

    clear_input_buffer();

    loop {
        if poll(Duration::from_millis(1000)).unwrap() {
            let event = read().unwrap();

            match event {
                Event::Key(key) => {
                    if !key.is_release() {
                        ()
                    } else {
                        match key.code {
                            KeyCode::Enter => {
                                break;
                            }
                            KeyCode::Tab => {
                                println!("tab");
                            }
                            KeyCode::Backspace => {
                                // todo fix this currently not working
                                input_buffer.pop();
                                print!("\r{} ", input_buffer);
                            }
                            code => {
                                print!("{}", code.as_char().unwrap_or(' '));
                                input_buffer.push(code.as_char().unwrap_or(' '));
                            }
                        }
                    }
                }
                _ => println!("dont care"),
            }

            std::io::stdout().flush().unwrap();
        } else {
            // duration expired
        }
    }

    println!("exiting");
    todo!()
}

fn clear_input_buffer() {
    while poll(std::time::Duration::from_millis(1)).unwrap() {
        let _ = read();
    }
}
