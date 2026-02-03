# Idea
a simple shell in Rust 

# Config

    all configs are writen in `shell.toml`

## Options

// give your shell an special look
[theme]
/*
colors:
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default, // no special color invisible in the terminal
*/
fg = "red"
bg = "default"

// how is your input line supposed to look like
[prompt]
lines = [[
    "{username}@{hostname} {cwd}",
    "{git}{pad:10}{ram}/{cpu}",
    "{time}",
]]

[prompt.username]
enabled = true
fg = ""
bg = ""
format = "{username}"

[prompt.hostname]
enabled = true
fg = "blue"
bg = ""
format = "{hostname}"

[prompt.cwd]
enabled = true
fg = "cyan"
bg = ""
format = "{cwd}"

[prompt.git]
enabled = true
fg = "yellow"
bg = ""
format = "{branch}{dirty}"

[prompt.ram]
enabled = true
fg = "magenta"
bg = ""
format = "{used}/{total}MB"

[prompt.cpu]
enabled = true
fg = "magenta"
bg = ""
format = "{usage}%"

[prompt.time]
enabled = true
fg = "white"
bg = ""
format = "{hh}:{mm}:{ss}"


allow for multiple configs for the things but only one can be activated
example:

[prompt.cwd]
active = "mycolor"

[prompt.cwd.normal]
fg = "cyan"

[prompt.cwd.mycolor]
fg = "magenta"

[prompt.cwd.minimal]
fg = "white"
