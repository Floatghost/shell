use crate::{
    config::{Color, ColorValue},
    parser::parse,
    userinput::TokenType,
};

pub struct StyledBlock {
    pub text: String,
    pub color_fg: ColorValue,
    pub color_bg: ColorValue,
}

pub fn highlight(buffer: &str) -> Vec<StyledBlock> {
    let tokens = parse(buffer);

    let mut styled = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        let mut styled_block = StyledBlock {
            text: token.raw.clone(),
            color_fg: ColorValue::Named(Color::White),
            color_bg: ColorValue::Named(Color::Default),
        };

        match token {
            _exit if token.raw.to_lowercase() == "exit" => {
                styled_block.color_fg = ColorValue::Named(Color::Green);
            }
            _quote if token.token_type == TokenType::Quote => {
                styled_block.color_fg = ColorValue::Named(Color::Cyan);
            }
            _arg if token.token_type == TokenType::Argument => {
                styled_block.color_fg = ColorValue::Named(Color::Magenta);
            }
            _flag if token.token_type == TokenType::Flag => {
                styled_block.color_fg = ColorValue::Named(Color::Gray);
            }
            _first if index == 0 => {
                styled_block.color_fg = ColorValue::Named(Color::Yellow);
            }
            _ => (),
        }

        styled.push(styled_block);

        if index + 1 != tokens.len() {
            styled.push(StyledBlock {
                text: " ".to_string(),
                color_fg: ColorValue::Named(Color::White),
                color_bg: ColorValue::Named(Color::Default),
            });
        }
    }

    styled
}
