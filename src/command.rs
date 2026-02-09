use std::{
    env,
    io::stdout,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};

use crossterm::{ExecutableCommand, cursor::MoveTo, terminal::Clear};
use phf::phf_map;
use rand::Rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{app::State, config::ShellConfig, parser::parse};

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

        let exec = &tokens[0].raw;
        let args: Vec<String> = tokens[1..].iter().map(|t| t.raw.to_string()).collect();

        if let Some(run) = find_in_builtin(exec) {
            return Some(Command {
                runner: run,
                args: args.to_vec(),
            });
        }
        if let Some(path) = find_in_path(exec) {
            return Some(Command {
                runner: path,
                args: args.to_vec(),
            });
        }
        if let Some(path) = find_in_env(exec) {
            return Some(Command {
                runner: Runner::Executable { path },
                args: args.to_vec(),
            });
        }

        None
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
                        "{:<6}  {:<15}  {:>8}  {}",
                        "Mode", "LastWriteTime", "Length", "Name"
                    );
                    println!(
                        "{:<6}  {:<15}  {:>8}  {}",
                        "----", "-------------", "------", "----"
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
                    if self.args.len() < 1 {
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
            },
        }
    }
}

/// find_in_env never returns an interpreted since there may be garbage in the path since
/// programs dont expect the shell to just exec random files
pub fn find_in_env(bin_name: &str) -> Option<PathBuf> {
    let path_var = match env::var("PATH") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to read PATH: {}", e);
            return None;
        }
    };

    for dir in env::split_paths(&path_var) {
        let full_path = dir.join(bin_name);

        #[cfg(windows)]
        let full_path = {
            if full_path.extension().is_none() {
                full_path.with_extension("exe")
            } else {
                full_path
            }
        };

        if full_path.is_file() && full_path.extension().unwrap().to_str().unwrap() == "exe" {
            return Some(full_path);
        }
    }

    None
}

pub fn find_in_path(bin_name: &str) -> Option<Runner> {
    let current_dir = env::current_dir().unwrap();

    let full_path = current_dir.join(bin_name);

    #[cfg(windows)]
    let full_path = {
        if full_path.extension().is_none() {
            full_path.with_extension("exe")
        } else {
            full_path
        }
    };

    #[allow(unused)]
    if full_path.is_file() {
        match full_path.extension().unwrap().to_str().unwrap() {
            "exe" => Some(Runner::Executable { path: full_path }),
            inter => {
                // allow the user to pass interpreters and an regex match
                // and then pass all files to the interpreters if they just get called as an path
                todo!()
            }
            _ => None,
        }
    } else {
        None
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
};

pub fn find_in_builtin(bin_name: &str) -> Option<Runner> {
    Some(Runner::InBuilt(
        BuiltIn::from(&bin_name.to_lowercase())?.clone(),
    ))
}

pub fn malloc_stress_process(dur: Duration) {
    let mut v = Vec::<u8>::new();

    if cfg!(debug_assertions) {
        for _ in 0..50 {
            v.extend(std::iter::repeat(0u8).take(1024 * 1024)); // 1 MB
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
