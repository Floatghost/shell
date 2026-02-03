pub fn tokenize(input: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current = String::new();
    let mut in_quotes: Option<char> = None;

    while let Some(c) = chars.next() {
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

pub fn group_commands(tokens: &Vec<String>) -> Vec<Vec<CommandAndArgs>> {
    let mut out: Vec<Vec<CommandAndArgs>> = Vec::new();
    let mut temp_commands: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for x in tokens {
        if x == "&" && !current.is_empty() {
            temp_commands.push(current);
            current = Vec::new();
        } else if x != "&" {
            current.push(x);
        }
    }

    if !current.is_empty() {
        temp_commands.push(current);
    }

    for comm in &temp_commands {
        out.push(construct_command(comm));
    }

    out
}

fn construct_command(input: &Vec<&str>) -> Vec<CommandAndArgs> {
    let mut temp: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for tok in input {
        if tok.to_string() == "|" && !current.is_empty() {
            temp.push(current);
            current = Vec::new();
        } else if tok.to_string() != "|" {
            current.push(tok);
        }
    }

    if !current.is_empty() {
        temp.push(current);
    }

    let out: Vec<CommandAndArgs> = temp
        .iter()
        .map(|part| CommandAndArgs {
            command_and_args: part.iter().map(|s| s.to_string()).collect(),
        })
        .collect();

    out
}
