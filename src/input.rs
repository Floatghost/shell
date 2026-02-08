use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, poll, read};

#[derive(Debug, PartialEq, Eq)]
pub enum EditorCommand {
    InsertChar(char),
    DeleteBackwards, // backspace
    DeleteForwards,  // delete key
    MoveLeft,
    MoveRight,
    MoveHome,
    MoveEnd,
    Tab, // tab-complete
    Enter,
    HistoryPrev,
    HistoryNext,
    Paste(String),
    NoOp, // all unused events
}

pub fn get_event() -> std::io::Result<EditorCommand> {
    if !poll(Duration::from_millis(50))? {
        return Ok(EditorCommand::NoOp);
    }

    match read()? {
        crossterm::event::Event::Key(key) => {
            // discard non press events
            if !key.is_press() {
                return Ok(EditorCommand::NoOp);
            }

            use EditorCommand as EC;

            return Ok(match key {
                #[allow(unused)]
                KeyEvent {
                    code,
                    modifiers,
                    state,
                    kind,
                } => match code {
                    KeyCode::Tab => EC::Tab,
                    KeyCode::Enter => EC::Enter,
                    KeyCode::Char(c) => EC::InsertChar(c),
                    KeyCode::Backspace => EC::DeleteBackwards,
                    KeyCode::Delete => EC::DeleteForwards,
                    KeyCode::Left => EC::MoveLeft,
                    KeyCode::Right => EC::MoveRight,
                    KeyCode::Home => EC::MoveHome,
                    KeyCode::End => EC::MoveEnd,
                    KeyCode::Up => EC::HistoryPrev,
                    KeyCode::Down => EC::HistoryNext,
                    _ => EC::NoOp,
                },
            });
        }
        crossterm::event::Event::Paste(pasting) => {
            return Ok(EditorCommand::Paste(pasting));
        }
        _ => return Ok(EditorCommand::NoOp),
    }
}
