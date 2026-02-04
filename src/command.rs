use std::{env, fs, path::PathBuf, process::Stdio};

use phf::phf_map;

use crate::app::State;

pub struct Command {
    pub runner: Runner,
    pub args: Vec<String>,
}

pub enum Runner {
    Executable { path: PathBuf },
    Interpreted { path: PathBuf, executor: PathBuf },
    InBuilt(BuiltIn),
}

impl Command {
    pub fn new(exec: &str, args: &[String]) -> Option<Command> {
        if let Some(com) = find_in_builtin(&exec) {
            return Some(Command {
                runner: Runner::InBuilt(com.clone()),
                args: args.to_vec(),
            });
        }
        if let Some(path) = find_in_env(&exec) {
            return Some(Command {
                runner: Runner::Executable { path: path },
                args: args.to_vec(),
            });
        }
        if let Some(path) = find_in_path(&exec) {
            return Some(Command {
                runner: Runner::Executable { path: path },
                args: args.to_vec(),
            });
        }

        None
    }

    pub fn exec(&self, state: &mut State) -> Option<i32> {
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
                            .and_then(|t| {
                                let dt = chrono::DateTime::<Local>::from(t);
                                Some(dt.format("%-d/%-m/%Y  %H:%M").to_string())
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
                    todo!()
                }
            },
        }
    }
}

pub fn find_in_env(bin_name: &str) -> Option<PathBuf> {
    let path_var = match env::var("PATH") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to read PATH: {}", e);
            return None;
        }
    };

    for dir in env::split_paths(&path_var) {
        let full_path = dir.join(&bin_name);

        // On Windows, executables may need ".exe" appended
        #[cfg(windows)]
        let full_path = {
            if full_path.extension().is_none() {
                full_path.with_extension("exe")
            } else {
                full_path
            }
        };

        if full_path.is_file() {
            println!("Found binary at: {}", full_path.display());
            return Some(full_path);
        }
    }

    println!("Binary '{}' not found in PATH", bin_name);
    None
}

pub fn find_in_path(bin_name: &str) -> Option<PathBuf> {
    let current_dir = env::current_dir().unwrap();

    let full_path = current_dir.join(&bin_name);

    // On Windows, executables may need ".exe" appended
    #[cfg(windows)]
    let full_path = {
        if full_path.extension().is_none() {
            full_path.with_extension("exe")
        } else {
            full_path
        }
    };

    if full_path.is_file() {
        Some(full_path)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltIn {
    Exit,
    Cd,
    PWD, // print working dir
    Ls,
    Where,
}

pub static BUILTIN_COMMANDS: phf::Map<&'static str, BuiltIn> = phf_map! {
    "exit" => BuiltIn::Exit,
    "cd"   => BuiltIn::Cd,
    "pwd" => BuiltIn::PWD,
    "ls" => BuiltIn::Ls,
    "where" => BuiltIn::Where,
};

pub fn find_in_builtin(bin_name: &str) -> Option<&BuiltIn> {
    BUILTIN_COMMANDS.get(&bin_name.to_lowercase())
}
