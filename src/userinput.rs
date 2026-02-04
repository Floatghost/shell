use std::{path::PathBuf, time::Duration};

use crossterm::{
    cursor::position,
    event::{poll, read, Event, KeyCode},
};

pub enum UserInput {
    Path(PathBuf),
    Ident(String),
}

pub fn get_userinput() -> Option<UserInput> {
    loop {
        if poll(Duration::from_millis(1000)).unwrap() {
            let event = read().unwrap();

            println!("Event::{event:?}\r");

            if event == Event::Key(KeyCode::Char('c').into()) {
                println!("Cursor position: {:?}\r", position());
            }

            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        } else {
            // duration expired
            println!(".");
        }
    }

    todo!()
}
