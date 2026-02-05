pub fn tokenize(input: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes: Option<char> = None;

    for c in input.chars() {
        match c {
            ' ' if in_quotes.is_none() => {
                if !current.is_empty() {
                    out.push(current);
                    current = String::new();
                }
            }
            '\'' | '"' => {
                if let Some(q) = in_quotes {
                    if q == c {
                        in_quotes = None;
                        out.push(current);
                        current = String::new();
                    } else {
                        current.push(c);
                    }
                } else {
                    in_quotes = Some(c);
                }
            }
            _ => {
                current += &c.to_string();
            }
        }
    }

    if !current.is_empty() {
        out.push(current);
    }

    out
}

/// in the command field the usual command is entered for example
/// find .txt
/// in the input field you can put an other command or an file so in this example it could be
/// ls -a
///
/// other example
/// command: output.txt
/// input: {
///     command: convert_to_binary
///     input: input.txt
/// }
#[derive(Debug)]
#[allow(unused)]
pub struct CommandAndArgs {
    pub command_and_args: Vec<String>,
}
