use std::{
    io::{Write, stdout},
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read},
    style,
};

use crate::{
    find::{Found, complete_command, complete_path},
    userinput::{Token, tokenize},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ReadLine {
    cursor_x: u16,
    start_pos: (u16, u16),
    buffer: String,
    prompt: String,
}

impl ReadLine {
    pub fn new(last_prompt_line: &str) -> ReadLine {
        ReadLine {
            cursor_x: 0,
            start_pos: (0, 0),
            buffer: String::new(),
            prompt: last_prompt_line.to_string(),
        }
    }

    pub fn get_line(&mut self) -> Result<String, String> {
        self.buffer = String::new();
        self.start_pos = cursor::position().unwrap();
        self.cursor_x = 0;

        ReadLine::clear_input_buffer();

        #[allow(unused)]
        loop {
            if poll(Duration::from_millis(50)).unwrap() {
                match read().unwrap() {
                    Event::Key(key) => {
                        if !key.is_press() {
                            match key {
                                KeyEvent {
                                    code,
                                    modifiers,
                                    kind,
                                    state,
                                } => {
                                    if let KeyCode::Char(c) = code
                                        && modifiers.contains(KeyModifiers::CONTROL)
                                    {
                                        return Err("ctrl-c".into());
                                    }
                                }
                            }

                            continue;
                        }

                        match key {
                            KeyEvent {
                                code,
                                modifiers,
                                state,
                                kind,
                            } => match code {
                                KeyCode::Tab => {
                                    // parse buffer and give back tokens + their offsets
                                    let tokens = tokenize(&self.buffer);

                                    if tokens.is_empty() {
                                        continue;
                                    }

                                    // println!();
                                    // dbg!(&tokens);

                                    // if the cursor is on a token or just after it start with autocomplete
                                    let focused = self.get_focused_token(&tokens);

                                    // println!("focused: {:?}", focused);

                                    let mut results = Vec::new();

                                    results.extend_from_slice(&complete_command(&focused.raw));
                                    results.extend_from_slice(&complete_path(&focused.raw));

                                    // merge all results if only one result is found replace the token with the result
                                    // assuming it changed

                                    if results.len() == 1 {
                                        self.overwrite_token(
                                            &focused,
                                            match &results[0] {
                                                Found::Path(p) => p.to_str().unwrap(),
                                                Found::EnvPath(envp) => {
                                                    envp.file_name().unwrap().to_str().unwrap()
                                                }
                                            },
                                        );
                                    } else if results.len() == 0 {
                                    } else {
                                        results.sort_by(|a, b| {
                                            a.file_name().len().cmp(&b.file_name().len())
                                        });
                                        self.overwrite_token(
                                            &focused,
                                            match &results[0] {
                                                Found::Path(p) => p.to_str().unwrap(),
                                                Found::EnvPath(envp) => {
                                                    envp.file_name().unwrap().to_str().unwrap()
                                                }
                                            },
                                        );
                                    }

                                    // other wise dont do anything
                                }
                                KeyCode::Enter => {
                                    println!();
                                    return Ok(self.buffer.clone());
                                }
                                KeyCode::Char(c) => {
                                    if modifiers.contains(KeyModifiers::CONTROL) {
                                        return Err("ctrl-c".into());
                                    }
                                    self.insert(c);
                                }
                                KeyCode::Backspace => {
                                    self.remove();
                                }
                                KeyCode::Left => {
                                    self.cursor_x = self.cursor_x.saturating_sub(1);
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(
                                        self.start_pos.0 + self.cursor_x,
                                        self.start_pos.1,
                                    ))
                                    .unwrap();
                                }
                                KeyCode::Right => {
                                    self.cursor_x = self
                                        .cursor_x
                                        .saturating_add(1)
                                        .min((self.buffer.len()) as u16);
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(
                                        self.start_pos.0 + self.cursor_x,
                                        self.start_pos.1,
                                    ))
                                    .unwrap();
                                }
                                KeyCode::Home => {
                                    self.cursor_x = 0;
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(self.start_pos.0, self.start_pos.1))
                                        .unwrap();
                                }
                                KeyCode::End => {
                                    self.cursor_x = self.buffer.len() as u16;
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(
                                        self.cursor_x + self.start_pos.0,
                                        self.start_pos.1,
                                    ))
                                    .unwrap();
                                }
                                _ => (),
                            },
                        }
                    }
                    event => {
                        // println!("event: {:?}", event);
                    }
                }
            }
        }
    }

    fn insert(&mut self, c: char) {
        // we dont need to clear the line since the new line is going to be bigger
        self.buffer.insert(self.cursor_x as usize, c);
        self.cursor_x += 1;

        let mut out = stdout();

        out.queue(cursor::MoveTo(
            self.cursor_x - 1 + self.start_pos.0,
            self.start_pos.1,
        ))
        .unwrap();

        let redraw_point = (self.cursor_x - 1) as usize;
        out.queue(style::Print(&self.buffer[redraw_point..]))
            .unwrap();

        out.queue(cursor::MoveTo(
            self.cursor_x + self.start_pos.0,
            self.start_pos.1,
        ))
        .unwrap();

        out.flush().unwrap();
    }

    fn remove(&mut self) {
        if self.cursor_x == 0 {
            // only delete chars that are before the bar
            return;
        }
        self.buffer.remove(self.cursor_x as usize - 1);
        self.cursor_x -= 1;

        let mut out = stdout();

        let buffer_redraw_point = self.cursor_x;
        let terminal_redraw_point = buffer_redraw_point + self.start_pos.0;

        out.queue(cursor::MoveTo(terminal_redraw_point, self.start_pos.1))
            .unwrap();

        out.queue(style::Print(&self.buffer[buffer_redraw_point as usize..]))
            .unwrap();

        // clear the deleted char
        out.queue(style::Print(" ")).unwrap();

        out.queue(cursor::MoveTo(
            self.cursor_x + self.start_pos.0,
            self.start_pos.1,
        ))
        .unwrap();

        out.flush().unwrap();
    }

    fn clear_input_buffer() {
        while poll(std::time::Duration::from_millis(1)).unwrap() {
            let _ = read();
        }
    }

    fn get_focused_token(&self, tokens: &[Token]) -> Token {
        let cx = self.cursor_x as usize;

        for token in tokens {
            let start = token.offset_x as usize;
            let end = start + token.raw.len();

            if cx >= start && cx <= end {
                return token.clone();
            }
        }

        if cx == self.buffer.len()
            && let Some(last) = tokens.last()
        {
            println!("in saftey");
            return last.clone();
        }

        Token {
            raw: "".to_string(),
            offset_x: cx as u32,
            offset_y: 0,
        }
    }

    fn overwrite_token(&mut self, t: &Token, replacing: &str) {
        let start = t.offset_x as usize;
        let end = start + t.raw.len();

        let old_len = t.raw.len();
        let new_len = replacing.len();

        self.buffer = format!(
            "{}{}{}",
            &self.buffer[..start],
            replacing,
            &self.buffer[end..],
        );

        let mut out = stdout();
        out.queue(cursor::MoveTo(
            self.start_pos.0 + start as u16,
            self.start_pos.1,
        ))
        .unwrap();

        out.queue(style::Print(&self.buffer[start..])).unwrap();

        if new_len < old_len {
            let diff = old_len - new_len;
            out.queue(style::Print(" ".repeat(diff))).unwrap();
        }

        let new_cursor = start + new_len;
        self.cursor_x = new_cursor as u16;

        out.queue(cursor::MoveTo(
            self.start_pos.0 + self.cursor_x,
            self.start_pos.1,
        ))
        .unwrap();

        out.flush().unwrap();
    }
}
