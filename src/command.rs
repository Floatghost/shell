use std::{
    io::{Write, stdout},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};

use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::{self, MoveTo},
    event::{Event, KeyCode, poll, read},
    style::{self, SetForegroundColor},
    terminal::{self, Clear},
};
use phf::phf_map;
use rand::{Rng, rng, seq::IndexedRandom};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    app::State,
    config::{ColorValue, ShellConfig},
    find::{self, Found},
    parser::parse,
};

pub struct Command {
    pub runner: Runner,
    pub args: Vec<String>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Runner {
    Executable { path: PathBuf },
    Interpreted { path: PathBuf, executor: PathBuf },
    InBuilt(BuiltIn),
}

impl Command {
    pub fn new(command: &str) -> Option<Command> {
        let tokens = parse(command);

        if tokens.is_empty() {
            return None;
        }

        let exec = &tokens[0].raw;
        let args: Vec<String> = tokens[1..].iter().map(|t| t.raw.to_string()).collect();

        match find::resolve(exec) {
            Some(Found::BuiltIn(b)) => Some(Command {
                runner: Runner::InBuilt(b),
                args,
            }),
            Some(Found::Path(p)) | Some(Found::EnvPath(p)) => Some(Command {
                runner: Runner::Executable { path: p },
                args,
            }),
            None => None,
        }
    }

    #[allow(unused)]
    pub fn exec(&self, state: &mut State, config: &mut ShellConfig) -> Option<i32> {
        match &self.runner {
            Runner::Executable { path } => {
                let mut cmd = std::process::Command::new(path);
                cmd.args(&self.args);

                cmd.stdout(Stdio::inherit());

                match cmd.spawn() {
                    Ok(mut child) => {
                        let status = child.wait().expect("failed to wait on child");
                        status.code()
                    }
                    Err(e) => {
                        eprintln!("Failed to execute command: {}", e);
                        None
                    }
                }
            }
            Runner::Interpreted { path, executor } => {
                todo!()
            }
            Runner::InBuilt(com) => match com {
                BuiltIn::Exit => {
                    todo!()
                }
                BuiltIn::Cd => {
                    state.cd(&self.args[0]).unwrap();
                    None
                }
                BuiltIn::PWD => {
                    println!("PWD: {}", state.cwd.to_str().unwrap());
                    None
                }
                BuiltIn::Ls => {
                    use chrono::Local;
                    use std::fs;

                    println!(
                        "{:<6}  {:<15}  {:>8}  Name",
                        "Mode", "LastWriteTime", "Length"
                    );
                    println!(
                        "{:<6}  {:<15}  {:>8}  ----",
                        "----", "-------------", "------"
                    );

                    for entry in fs::read_dir(&state.cwd).unwrap() {
                        let Ok(entry) = entry else { continue };
                        let Ok(metadata) = entry.metadata() else {
                            continue;
                        };

                        let is_dir = metadata.is_dir();
                        let entry_name = entry.file_name();
                        let name = entry_name.to_string_lossy();

                        let mode = if is_dir { "d-----" } else { "-a----" };

                        let modified = metadata.modified().ok();
                        let time_string = modified
                            .map(|t| {
                                let dt = chrono::DateTime::<Local>::from(t);
                                dt.format("%-d/%-m/%Y  %H:%M").to_string()
                            })
                            .unwrap_or_else(|| "".into());

                        let length = if is_dir {
                            "".into()
                        } else {
                            metadata.len().to_string()
                        };

                        println!("{:<6}  {:<15}  {:>8}  {}", mode, time_string, length, name);
                    }

                    None
                }
                BuiltIn::Where => {
                    if self.args.is_empty() {
                        return Some(1);
                    }
                    let query = match Command::new(&self.args.join(" ")) {
                        Some(com) => com,
                        None => return Some(1),
                    };

                    match query.runner {
                        Runner::Executable { path } => {
                            println!("exe @ {}", path.to_str().unwrap());
                        }
                        Runner::InBuilt(b) => {
                            println!("builtin command: {:?}", b);
                        }
                        Runner::Interpreted { path, executor } => {
                            println!(
                                "{} interpreted by {}",
                                path.to_str().unwrap(),
                                executor.to_str().unwrap()
                            );
                        }
                    }

                    None
                }
                BuiltIn::Clear => {
                    let mut out = stdout();

                    out.execute(Clear(crossterm::terminal::ClearType::All));
                    out.execute(MoveTo(0, 0));

                    None
                }
                BuiltIn::Sigma | BuiltIn::NixOsSig => {
                    for i in 0..1000 {
                        println!("Sigma 🐺🗿");
                    }

                    None
                }
                BuiltIn::Malloc => {
                    std::process::Command::new(std::env::current_exe().unwrap())
                        .arg("--malloc-stress")
                        // CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
                        .creation_flags(0x00000008 | 0x08000000 | 0x00000200)
                        .spawn()
                        .expect("uwu help");

                    None
                }
                BuiltIn::Garbage => {
                    println!("🤮 WRONG.\nYOU'RE WRONG.\nTAKE THIS🥊");

                    std::process::Command::new(std::env::current_exe().unwrap())
                        .arg("--malloc-stress")
                        // CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
                        .creation_flags(0x00000008 | 0x08000000 | 0x00000200)
                        .spawn()
                        .expect("uwu help");

                    None
                }
                BuiltIn::Reload => {
                    let conf = ShellConfig::new().unwrap();
                    *config = conf;
                    None
                }
                BuiltIn::EnvList => {
                    let args = &self.args;

                    if args.is_empty() {
                        // list all envs
                        for (variable, values) in std::env::vars() {
                            println!("{}: {}", variable, values);
                        }
                    } else {
                        for arg in args {
                            if let Ok(values) = std::env::var(arg) {
                                println!("{}: {}", arg, values);
                            }
                        }
                    }

                    None
                }
                BuiltIn::Help => {
                    for command in BuiltIn::iter() {
                        let aliases = command.aliases().join(", ");
                        println!("{}", aliases);
                    }

                    None
                }
                BuiltIn::Crazy => {
                    use crossterm::{
                        ExecutableCommand,
                        cursor::MoveTo,
                        event::{Event, KeyCode, poll, read},
                        style,
                    };
                    use rand::{random, rng};
                    use std::time::Duration;

                    let mut out = stdout();
                    let (x, y) = terminal::size().unwrap();
                    let mut rng = rng();

                    // Infinite loop – press 'q' to exit
                    loop {
                        // Check keyboard input every 0ms (non-blocking)
                        if poll(Duration::from_millis(0)).unwrap() {
                            if let Event::Key(key) = read().unwrap() {
                                if key.code == KeyCode::Char('q') {
                                    break;
                                }
                            }
                        }

                        let color =
                            ColorValue::Hex(format!("#{:06X}", rng.random_range(0..=0xFFFFFF)));

                        out.execute(MoveTo(random::<u16>() % x, random::<u16>() % y))
                            .unwrap();
                        out.execute(style::Print(format!("{} ", color.to_ansi_bg())))
                            .unwrap();
                    }

                    None
                }
                BuiltIn::Matrix => {
                    let colored = self.args.iter().any(|a| a == "color");
                    let mut out = stdout();
                    let (w, h) = terminal::size().unwrap();
                    let mut rng = rng();

                    let trail_len: u16 = 15;
                    let total_dist = h + trail_len;

                    #[derive(Clone, Copy)]
                    struct StreamColor {
                        r: u8,
                        g: u8,
                        b: u8,
                    }

                    let mut drops: Vec<i32> = (0..w)
                        .map(|x| {
                            let stagger = (x as f32 / w as f32 * total_dist as f32) as i32;
                            0 - stagger
                        })
                        .collect();

                    use rand::seq::SliceRandom;
                    drops.shuffle(&mut rng);

                    let mut colors: Vec<StreamColor> = (0..w)
                        .map(|_| {
                            if colored {
                                StreamColor {
                                    r: rng.random(),
                                    g: rng.random(),
                                    b: rng.random(),
                                }
                            } else {
                                StreamColor { r: 0, g: 255, b: 0 }
                            }
                        })
                        .collect();

                    let charset: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@$%&*+=-|<>!?:;";

                    out.queue(cursor::Hide).unwrap();

                    loop {
                        if poll(Duration::from_millis(0)).unwrap() {
                            if let Event::Key(k) = read().unwrap() {
                                if k.code == KeyCode::Char('q') {
                                    break;
                                }
                            }
                        }

                        for x in 0..w {
                            let head_y = drops[x as usize];
                            let base = colors[x as usize];

                            if head_y >= 0 && head_y < h as i32 {
                                let head_char = *charset.choose(&mut rng).unwrap() as char;
                                out.queue(SetForegroundColor(style::Color::White)).unwrap();
                                out.queue(MoveTo(x, head_y as u16)).unwrap();
                                out.queue(style::Print(head_char)).unwrap();
                            }

                            for i in 1..=trail_len {
                                let y_signed = head_y - i as i32;
                                if y_signed >= 0 && y_signed < h as i32 {
                                    let y = y_signed as u16;
                                    let ch = *charset.choose(&mut rng).unwrap() as char;
                                    let fade = 1.0 - (i as f32 / trail_len as f32);

                                    out.queue(SetForegroundColor(style::Color::Rgb {
                                        r: (base.r as f32 * fade) as u8,
                                        g: (base.g as f32 * fade) as u8,
                                        b: (base.b as f32 * fade) as u8,
                                    }))
                                    .unwrap();
                                    out.queue(MoveTo(x, y)).unwrap();
                                    out.queue(style::Print(ch)).unwrap();
                                }
                            }

                            let clear_y = head_y - trail_len as i32;
                            if clear_y >= 0 && clear_y < h as i32 {
                                out.queue(MoveTo(x, clear_y as u16)).unwrap();
                                out.queue(style::Print(" ")).unwrap();
                            }

                            if head_y >= (h + trail_len) as i32 {
                                drops[x as usize] = 0;
                                if colored {
                                    colors[x as usize] = StreamColor {
                                        r: rng.random(),
                                        g: rng.random(),
                                        b: rng.random(),
                                    };
                                }
                            } else {
                                drops[x as usize] += 1;
                            }
                        }

                        out.flush().unwrap();
                        std::thread::sleep(Duration::from_millis(20));
                    }

                    out.queue(cursor::Show).unwrap();
                    None
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum BuiltIn {
    Exit,
    Cd,
    PWD, // print working dir
    Ls,
    Where,
    Clear,
    Sigma,
    Malloc,
    Garbage,
    NixOsSig,
    Reload,
    EnvList,
    Help,
    Crazy,
    Matrix,
}

impl BuiltIn {
    pub fn aliases(&self) -> &'static [&'static str] {
        match self {
            Self::Exit => &["exit"],
            Self::Cd => &["cd"],
            Self::PWD => &["pwd"],
            Self::Ls => &["ls"],
            Self::Where => &["where"],
            Self::Clear => &["clear"],
            Self::Sigma => &["🐺🗿", "sigma"],
            Self::Malloc => &["malloc"],
            Self::Garbage => &["nixos<<"],
            Self::NixOsSig => &["nixos>>"],
            Self::Reload => &["reload"],
            Self::EnvList => &["$env", "env"],
            Self::Help => &["help"],
            Self::Crazy => &["lsd"],
            Self::Matrix => &["matrix", "hacker"],
        }
    }

    pub fn to_string(&self) -> String {
        self.aliases()[0].to_string()
    }

    pub fn from(input: &str) -> Option<Self> {
        match input {
            "exit" => Some(BuiltIn::Exit),
            "cd" => Some(BuiltIn::Cd),
            "pwd" => Some(BuiltIn::PWD),
            "ls" => Some(BuiltIn::Ls),
            "where" => Some(BuiltIn::Where),
            "clear" => Some(BuiltIn::Clear),
            "sigma" => Some(BuiltIn::Sigma),
            "🐺🗿" => Some(BuiltIn::Sigma),
            "malloc" => Some(BuiltIn::Malloc),
            "nixos>>" => Some(BuiltIn::NixOsSig),
            "nixos<<" => Some(BuiltIn::Garbage),
            "reload" => Some(BuiltIn::Reload),
            "help" => Some(BuiltIn::Help),
            "lsd" => Some(BuiltIn::Crazy),
            "matrix" => Some(BuiltIn::Matrix),
            "hacker" => Some(BuiltIn::Matrix),
            _ if input.starts_with("$env") | input.starts_with("env") => Some(BuiltIn::EnvList),
            _ => None,
        }
    }
}

pub static BUILTIN_COMMANDS: phf::Map<&'static str, BuiltIn> = phf_map! {
    "exit" => BuiltIn::Exit,
    "cd"   => BuiltIn::Cd,
    "pwd" => BuiltIn::PWD,
    "ls" => BuiltIn::Ls,
    "where" => BuiltIn::Where,
    "clear" => BuiltIn::Clear,
    "sigma" => BuiltIn::Sigma,
    "🐺🗿" => BuiltIn::Sigma,
    "malloc" => BuiltIn::Malloc,
    "nixos>>" => BuiltIn::NixOsSig,
    "nixos<<" => BuiltIn::Garbage,
    "reload" => BuiltIn::Reload,
    "$env" => BuiltIn::EnvList,
    "env" => BuiltIn::EnvList,
    "help" => BuiltIn::Help,
    "lsd" => BuiltIn::Crazy,
    "matrix" => BuiltIn::Matrix,
    "hacker" => BuiltIn::Matrix,
};

pub fn malloc_stress_process(dur: Duration) {
    let mut v = Vec::<u8>::new();

    if cfg!(debug_assertions) {
        for _ in 0..50 {
            v.extend(std::iter::repeat_n(0u8, 1024 * 1024)); // 1 MB
        }

        std::thread::sleep(dur);
        return;
    }

    let start = Instant::now();
    loop {
        if start.elapsed() >= dur {
            return;
        }
        let mut chunk = vec![0u8; 1024 * 1024]; // 1MB
        rand::rng().fill(&mut chunk[..]);
        v.extend_from_slice(&chunk);
    }
}
