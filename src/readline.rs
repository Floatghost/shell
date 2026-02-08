use std::{
    io::{Write, stdout},
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read},
    style,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    CTRLC_COUNT,
    config::{Color, ColorValue},
    find::{Found, complete_builtin, complete_command, complete_path},
    userinput::{Token, TokenType, tokenize},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ReadLine {
    cursor_x: u16,
    start_pos: (u16, u16),
    buffer: String,
    prompt: String,
    tab_cache: Option<Vec<Found>>,
    tab_clock: usize,
}

impl ReadLine {
    pub fn new(last_prompt_line: &str) -> ReadLine {
        ReadLine {
            cursor_x: 0,
            start_pos: (0, 0),
            buffer: String::new(),
            prompt: last_prompt_line.to_string(),
            tab_cache: None,
            tab_clock: 0,
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
                                        && c == 'c'
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
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                    let mut overwrite = String::new();

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

                                    if let Some(cached) = &self.tab_cache {
                                        if self.tab_clock == cached.len() {
                                            self.tab_clock = 0;
                                        }
                                        overwrite = cached[self.tab_clock].file_name();
                                        self.tab_clock += 1;
                                    } else {
                                        let mut results = Vec::new();

                                        results.extend_from_slice(&complete_builtin(&focused.raw));
                                        results.extend_from_slice(&complete_command(&focused.raw));
                                        results.extend_from_slice(&complete_path(&focused.raw));

                                        // merge all results if only one result is found replace the token with the result
                                        // assuming it changed

                                        if results.is_empty() {
                                            continue;
                                        }
                                        if results.len() > 1 {
                                            results.sort_by(|a, b| {
                                                a.file_name().len().cmp(&b.file_name().len())
                                            });
                                            self.tab_cache = Some(results.clone());
                                            self.tab_clock = 1;
                                        }

                                        overwrite = results[0].file_name();
                                    }
                                    self.overwrite_token(&focused, &overwrite);
                                    // dbg!(focused);
                                    // dbg!(overwrite);

                                    // other wise dont do anything
                                }
                                KeyCode::Enter => {
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                    println!();
                                    return Ok(self.buffer.clone());
                                }
                                KeyCode::Char(c) => {
                                    // if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                    //     return Err("ctrl-c".into());
                                    // }
                                    self.insert(c);
                                    self.tab_cache = None;
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                }
                                KeyCode::Backspace => {
                                    self.remove();
                                    self.tab_cache = None;
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                }
                                KeyCode::Left => {
                                    self.cursor_x = self.cursor_x.saturating_sub(1);
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(
                                        self.start_pos.0 + self.cursor_col(),
                                        self.start_pos.1,
                                    ))
                                    .unwrap();
                                    self.tab_cache = None;
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                }
                                KeyCode::Right => {
                                    let char_count = self.buffer.chars().count();
                                    self.cursor_x =
                                        self.cursor_x.saturating_add(1).min(char_count as u16);
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(
                                        self.start_pos.0 + self.cursor_col(),
                                        self.start_pos.1,
                                    ))
                                    .unwrap();
                                    self.tab_cache = None;
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                }
                                KeyCode::Home => {
                                    self.cursor_x = 0;
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(self.start_pos.0, self.start_pos.1))
                                        .unwrap();
                                    self.tab_cache = None;
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
                                }
                                KeyCode::End => {
                                    self.cursor_x = self.buffer.chars().count() as u16;
                                    let mut out = stdout();
                                    out.execute(cursor::MoveTo(
                                        self.cursor_col() + self.start_pos.0,
                                        self.start_pos.1,
                                    ))
                                    .unwrap();
                                    self.tab_cache = None;
                                    CTRLC_COUNT.store(1, std::sync::atomic::Ordering::SeqCst);
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

    fn cursor_byte_index(&self) -> usize {
        self.buffer
            .chars()
            .take(self.cursor_x as usize)
            .map(|c| c.len_utf8())
            .sum()
    }

    fn cursor_col(&self) -> u16 {
        let b = self.cursor_byte_index();
        UnicodeWidthStr::width(&self.buffer[..b]) as u16
    }

    fn col_at_byte(&self, byte_idx: usize) -> u16 {
        UnicodeWidthStr::width(&self.buffer[..byte_idx]) as u16
    }

    fn insert(&mut self, c: char) {
        // we dont need to clear the line since the new line is going to be bigger
        let byte_idx = self.cursor_byte_index();
        self.buffer.insert(byte_idx, c);
        self.cursor_x += 1;

        let mut out = stdout();

        out.queue(cursor::MoveTo(
            self.cursor_col() - 1 + self.start_pos.0,
            self.start_pos.1,
        ))
        .unwrap();

        out.queue(style::Print(&self.buffer[byte_idx..])).unwrap();

        out.queue(cursor::MoveTo(
            self.cursor_col() + self.start_pos.0,
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
        self.cursor_x -= 1;
        let byte_idx = self.cursor_byte_index();
        self.buffer.remove(byte_idx);

        let mut out = stdout();

        let buffer_redraw_point = byte_idx;
        let terminal_redraw_point = self.cursor_col() + self.start_pos.0;

        out.queue(cursor::MoveTo(terminal_redraw_point, self.start_pos.1))
            .unwrap();

        out.queue(style::Print(&self.buffer[buffer_redraw_point..]))
            .unwrap();

        // clear the deleted char
        out.queue(style::Print(" ")).unwrap();

        out.queue(cursor::MoveTo(
            self.cursor_col() + self.start_pos.0,
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
        let cx = self.cursor_byte_index();

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
            token_type: TokenType::Unknown,
            offset_x: cx as u32,
            offset_y: 0,
        }
    }

    fn overwrite_token(&mut self, t: &Token, replacing: &str) {
        let start = t.offset_x as usize;
        let end = start + t.raw.len();

        let old_tail = self.buffer.len() - end;

        self.buffer = format!(
            "{}{}{}",
            &self.buffer[..start],
            replacing,
            &self.buffer[end..],
        );

        let mut out = stdout();

        let start_col = self.col_at_byte(start);
        out.queue(cursor::MoveTo(
            self.start_pos.0 + start_col,
            self.start_pos.1,
        ))
        .unwrap();

        out.queue(style::Print(&self.buffer[start..])).unwrap();

        let clear_cols = UnicodeWidthStr::width(&self.buffer[self.buffer.len() - old_tail..]) + 4;
        out.queue(style::Print(" ".repeat(clear_cols))).unwrap();

        let new_cursor_byte = start + replacing.len();

        self.cursor_x = self.buffer[..new_cursor_byte].chars().count() as u16;

        let new_cursor_col = self.col_at_byte(new_cursor_byte);
        out.queue(cursor::MoveTo(
            self.start_pos.0 + new_cursor_col,
            self.start_pos.1,
        ))
        .unwrap();

        out.flush().unwrap();
    }
}
