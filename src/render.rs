use std::io::{Write, stdout};

use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};

use crate::highlight::StyledBlock;

#[derive(Debug, Clone, Copy)]
pub struct TerminalPos {
    pub x: u16,
    pub y: u16,
}

pub struct RenderEngine {
    current_prompt_pos: TerminalPos,
    current_prompt_buffer: Vec<StyledBlock>,
}

impl RenderEngine {
    pub fn new() -> RenderEngine {
        RenderEngine {
            current_prompt_pos: TerminalPos { x: 0, y: 0 },
            current_prompt_buffer: Vec::new(),
        }
    }

    pub fn render_new_prompt(&mut self, prompt: &str) {
        print!("{}", prompt);
        stdout().flush().unwrap();

        let (x, y) = cursor::position().unwrap();
        self.current_prompt_pos = TerminalPos { x, y };
    }

    pub fn render_current_prompt(&mut self, buffer: Vec<StyledBlock>, cursor_pos: TerminalPos) {
        let mut stdout = stdout();

        stdout
            .queue(cursor::MoveTo(
                self.current_prompt_pos.x,
                self.current_prompt_pos.y,
            ))
            .unwrap();

        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();

        for block in &buffer {
            print!(
                "{}{}{}\x1b[0m",
                block.color_fg.to_ansi_fg(),
                block.color_bg.to_ansi_bg(),
                block.text
            );
        }
        stdout.flush().unwrap();

        stdout
            .execute(cursor::MoveTo(
                self.current_prompt_pos.x + cursor_pos.x,
                self.current_prompt_pos.y + cursor_pos.y,
            ))
            .unwrap();
        stdout.flush().unwrap();

        self.current_prompt_buffer = buffer;
    }
}
