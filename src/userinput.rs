#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Command,
    Argument,
    Flag,     // --help, -v
    Operator, // |, >, &&
    Quote,    // "string"
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub raw: String,
    pub token_type: TokenType,
    pub offset_x: u32,
    pub offset_y: u32,
}

impl Token {
    pub fn new(x: u32, y: u32) -> Token {
        Token {
            raw: String::new(),
            token_type: TokenType::Unknown,
            offset_x: x,
            offset_y: y,
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut offset_x = 0;
    let mut offset_y = 0;

    let mut temp = Token::new(offset_x, offset_y);

    for c in input.chars() {
        if c == '\n' {
            if !temp.raw.is_empty() {
                temp.token_type = if out.is_empty() {
                    TokenType::Command
                } else {
                    TokenType::Argument
                };
                out.push(temp);
                temp = Token::new(0, offset_y + 1);
            }

            offset_y += 1;
            offset_x = 0;
            continue;
        }

        if c.is_whitespace() {
            if !temp.raw.is_empty() {
                temp.token_type = if out.is_empty() {
                    TokenType::Command
                } else {
                    TokenType::Argument
                };
                out.push(temp);
                temp = Token::new(offset_x + 1, offset_y);
            } else {
                temp.offset_x += 1;
            }
        } else {
            if temp.raw.is_empty() {
                temp.offset_x = offset_x;
                temp.offset_y = offset_y;
            }
            temp.raw.push(c);
        }

        offset_x += c.len_utf8() as u32;
    }

    if !temp.raw.is_empty() {
        temp.token_type = if out.is_empty() {
            TokenType::Command
        } else {
            TokenType::Argument
        };
        out.push(temp);
    }

    out
}
