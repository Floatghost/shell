use crate::{
    find::{self, Found},
    highlight::highlight,
    input::{self, EditorCommand},
    parser::parse,
    render::{RenderEngine, TerminalPos},
    userinput::{self, Token},
};

pub struct Editor {
    buffer: String,
    cursor_char_idx: usize,
    cursor_pos: TerminalPos,
    tab_cache: Option<Vec<Found>>,
    tab_clock: usize,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            buffer: String::new(),
            cursor_char_idx: 0,
            cursor_pos: TerminalPos { x: 0, y: 0 },
            tab_cache: None,
            tab_clock: 0,
        }
    }

    pub fn get_userinput(
        &mut self,
        renderer: &mut RenderEngine,
        prompt: &str,
        history: &[String],
    ) -> std::io::Result<String> {
        renderer.render_new_prompt(prompt);

        let mut history_clock = history.len();

        loop {
            let command_event = input::get_event()?;

            match command_event {
                EditorCommand::Enter => break,
                EditorCommand::NoOp => continue,
                _ => (),
            }

            self.apply(command_event, history, &mut history_clock);

            let highlighted = highlight(&self.buffer);

            renderer.render_current_prompt(highlighted, self.cursor_pos);
        }

        println!();
        Ok(self.buffer.to_string())
    }

    pub fn clear(&mut self) {
        self.buffer = String::new();
        self.cursor_char_idx = 0;
        self.cursor_pos = TerminalPos { x: 0, y: 0 };
    }

    fn get_byte_idx(&self) -> usize {
        self.buffer
            .chars()
            .take(self.cursor_char_idx)
            .map(|c| c.len_utf8())
            .sum()
    }

    fn get_visual_width(&self) -> usize {
        let byte_idx = self.get_byte_idx();

        let col = unicode_width::UnicodeWidthStr::width(&self.buffer[..byte_idx]);

        col
    }

    /// insert at the pos of the cursor
    fn insert(&mut self, c: char) {
        let insert_idx = self.get_byte_idx();

        self.buffer.insert(insert_idx, c);
    }

    fn insert_str(&mut self, s: &str) {
        let insert_idx = self.get_byte_idx();

        self.buffer.insert_str(insert_idx, s);
    }

    fn delete(&mut self, byte_idx: usize) {
        if byte_idx < self.buffer.len() {
            self.buffer.remove(byte_idx);
        }
    }

    fn get_focused_token(&self, tokens: &[Token]) -> Token {
        let cx = self.get_byte_idx();

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
            return last.clone();
        }

        Token {
            raw: "".to_string(),
            token_type: userinput::TokenType::Unknown,
            offset_x: cx as u32,
            offset_y: 0,
        }
    }

    fn overwrite_token(&mut self, t: &Token, replacing: &str) {
        let start = t.offset_x as usize;
        let end = start + t.raw.len();

        self.buffer = format!(
            "{}{}{}",
            &self.buffer[..start],
            replacing,
            &self.buffer[end..],
        );

        let new_cursor_byte = start + replacing.len();
        self.cursor_char_idx = self.buffer[..new_cursor_byte].chars().count();
    }

    fn apply(
        &mut self,
        command_event: EditorCommand,
        history: &[String],
        history_clock: &mut usize,
    ) {
        let mut clear_cache = true;

        match command_event {
            EditorCommand::InsertChar(c) => {
                self.insert(c);
                // todo support multiline
                self.cursor_char_idx += 1;
            }
            EditorCommand::DeleteBackwards => {
                if self.cursor_char_idx > 0 {
                    self.cursor_char_idx -= 1;
                    let idx = self.get_byte_idx();
                    self.delete(idx);
                }
            }
            EditorCommand::DeleteForwards => {
                let idx = self.get_byte_idx();
                self.delete(idx);
            }
            EditorCommand::MoveLeft => {
                if self.cursor_char_idx > 0 {
                    self.cursor_char_idx -= 1;
                }
            }
            EditorCommand::MoveRight => {
                if self.cursor_char_idx < self.buffer.chars().count() {
                    self.cursor_char_idx += 1;
                }
            }
            EditorCommand::MoveHome => {
                self.cursor_char_idx = 0;
            }
            EditorCommand::MoveEnd => {
                self.cursor_char_idx = self.buffer.chars().count();
            }
            EditorCommand::Tab => {
                clear_cache = false;
                let tokens = parse(&self.buffer);
                if !tokens.is_empty() {
                    let focused = self.get_focused_token(&tokens);
                    let overwrite = if let Some(cached) = &self.tab_cache {
                        if self.tab_clock >= cached.len() {
                            self.tab_clock = 0;
                        }
                        let res = cached[self.tab_clock].file_name();
                        self.tab_clock += 1;
                        res
                    } else {
                        let mut results = Vec::new();
                        results.extend(find::complete_builtin(&focused.raw));
                        results.extend(find::complete_command(&focused.raw));
                        results.extend(find::complete_path(&focused.raw));

                        if !results.is_empty() {
                            results.sort_by(|a, b| a.file_name().len().cmp(&b.file_name().len()));
                            let first = results[0].file_name();
                            if results.len() > 1 {
                                self.tab_cache = Some(results.clone());
                                self.tab_clock = 1;
                            }

                            // dont autocomplete already complete commands
                            if first == focused.raw {
                                if results.len() >= 2 {
                                    self.tab_clock = 2;
                                    results[1].file_name()
                                } else {
                                    return;
                                }
                            } else {
                                first
                            }
                        } else {
                            return;
                        }
                    };
                    self.overwrite_token(&focused, &overwrite);
                }
            }
            EditorCommand::Enter => return, // get_userinput should handle this
            EditorCommand::HistoryPrev => {
                if history.is_empty() {
                    return;
                }

                *history_clock = history_clock.saturating_sub(1);
                let new_prompt = history[*history_clock].clone();
                self.cursor_char_idx = new_prompt.chars().count();
                self.buffer = new_prompt;
            }
            EditorCommand::HistoryNext => {
                if history.is_empty() {
                    return;
                }

                if *history_clock + 1 > history.len() - 1 {
                    self.buffer = String::new();
                    self.cursor_char_idx = 0;
                } else {
                    *history_clock += 1;
                    let new_promt = history[*history_clock].clone();
                    self.cursor_char_idx = new_promt.chars().count();
                    self.buffer = new_promt;
                }
            }
            EditorCommand::Paste(pasting) => {
                self.insert_str(&pasting);
                // todo support multiline
                self.cursor_char_idx += pasting.chars().count();
            }
            EditorCommand::NoOp => return,
        }

        if clear_cache {
            self.tab_cache = None;
            self.tab_clock = 0;
        }

        self.cursor_pos.x = self.get_visual_width() as u16;
    }
}
