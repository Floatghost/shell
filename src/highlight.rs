use crate::{
    config::{Color, ColorValue},
    parser::parse,
    userinput::tokenize,
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
        if index == 0 {
            styled.push(StyledBlock {
                text: token.raw.clone(),
                color_fg: ColorValue::Named(Color::Yellow),
                color_bg: ColorValue::Named(Color::Default),
            });
        } else {
            styled.push(StyledBlock {
                text: token.raw.clone(),
                color_fg: ColorValue::Named(Color::White),
                color_bg: ColorValue::Named(Color::Default),
            });
        }

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
