use crate::userinput::{Token, tokenize};

// todo in the future also handle multiple commands and piping etc
pub fn parse(raw: &str) -> Vec<Token> {
    let tokens = tokenize(raw);

    tokens
}
