use crate::parse_input::*;
use std::{env, path::PathBuf};
use std::process::{Command, Stdio};

pub fn execute_command(command: Vec<Command_AND_ARGS>) -> Option<i32> {

    let mut output: String = String::new();
    

    for com in command {
        println!("executing command: {:?}", com);
        if let Some(bin_path) = search_path_for_bin(&com.command_and_args[0]) {
            let args = &com.command_and_args[1..]; // everything after the binary is an argument
            run_bin_with_args(bin_path, args);
        } else {
            eprintln!("Binary not found: {}", com.command_and_args[0]);
        }
    }
    println!("done executing commands");

    None
}

fn search_path_for_bin(bin_name: &str) -> Option<PathBuf> {
    let path_var = match env::var("PATH") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to read PATH: {}", e);
            return None;
        }
    };

    for dir in env::split_paths(&path_var) {
        let full_path = dir.join(bin_name);

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

fn run_bin_with_args(bin_path: PathBuf, args: &[String]) -> Option<i32> {
    let mut cmd = Command::new(bin_path);
    cmd.args(args);

    match cmd.status() {
        Ok(status) => {
            println!("Process exited with: {}", status);
            status.code() // returns Option<i32>
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            None
        }
    }
}
