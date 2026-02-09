#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Command,
    Argument,
    Flag,     // --help, -v
    Operator, // |, >, >>, <, &&, ||
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

    let mut t = Token::new(offset_x, offset_y);
    let mut in_quote = false;
    let mut quote_char: Option<char> = None;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        // Handle newline -> push token + reset
        if c == '\n' {
            if !t.raw.is_empty() {
                classify_token(&mut t, out.is_empty());
                out.push(t);
            }
            t = Token::new(0, offset_y + 1);
            offset_y += 1;
            offset_x = 0;
            continue;
        }

        // Handle quote entering/exiting
        if (c == '"' || c == '\'') && !in_quote {
            // Start of quoted string
            in_quote = true;
            quote_char = Some(c);
            t.offset_x = offset_x;
            t.raw.push(c);
            offset_x += c.len_utf8() as u32;
            continue;
        } else if Some(c) == quote_char && in_quote {
            // End of quoted string
            t.raw.push(c);
            in_quote = false;
            quote_char = None;
            offset_x += c.len_utf8() as u32;
            continue;
        }

        // If inside quotes: just append
        if in_quote {
            t.raw.push(c);
            offset_x += c.len_utf8() as u32;
            continue;
        }

        // Detect operators: |, ||, >, >>, <, &&, ||
        if is_operator_char(c) {
            if !t.raw.is_empty() {
                classify_token(&mut t, out.is_empty());
                out.push(t);
                t = Token::new(offset_x, offset_y);
            }

            let mut op = c.to_string();

            // Try two-char operator
            if let Some(&next) = chars.peek() {
                let two = format!("{}{}", c, next);
                if is_operator(&two) {
                    op = two;
                    chars.next(); // consume second char
                }
            }

            let mut op_token = Token::new(offset_x, offset_y);
            op_token.raw = op;
            op_token.token_type = TokenType::Operator;
            offset_x += op_token.raw.len() as u32;
            out.push(op_token);

            continue;
        }

        // Whitespace ends a token
        if c.is_whitespace() {
            if !t.raw.is_empty() {
                classify_token(&mut t, out.is_empty());
                out.push(t);
                t = Token::new(offset_x + 1, offset_y);
            } else {
                // skip whitespace
                t.offset_x += 1;
            }
            offset_x += 1;
            continue;
        }

        // Normal character
        if t.raw.is_empty() {
            t.offset_x = offset_x;
            t.offset_y = offset_y;
        }

        t.raw.push(c);
        offset_x += c.len_utf8() as u32;
    }

    // Final token
    if !t.raw.is_empty() {
        classify_token(&mut t, out.is_empty());
        out.push(t);
    }

    out
}

fn is_operator_char(c: char) -> bool {
    matches!(c, '|' | '&' | '>' | '<')
}

fn is_operator(s: &str) -> bool {
    matches!(s, "|" | "||" | "&" | "&&" | ">" | ">>" | "<")
}

fn classify_token(t: &mut Token, is_first: bool) {
    let raw = t.raw.as_str();

    if raw.starts_with('"') || raw.starts_with('\'') {
        t.token_type = TokenType::Quote;
        return;
    }

    if raw.starts_with("--") || (raw.starts_with('-') && raw.len() > 1) {
        t.token_type = TokenType::Flag;
        return;
    }

    if is_first {
        t.token_type = TokenType::Command;
        return;
    }

    t.token_type = TokenType::Argument;
}
