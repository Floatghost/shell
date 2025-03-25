use crate::parse_input::*;
use std::{env, path::PathBuf};
use std::process::{Command, Stdio};

pub fn execute_command(command: Vec<CommandAndArgs>) -> Option<i32> {
    if command.is_empty() {
        return None;
    }

    let mut previous_output: Option<std::process::ChildStdout> = None;

    for (i, com) in command.iter().enumerate() {
        println!("executing command: {:?}", com);
        let bin_name = &com.command_and_args[0];
        let args = &com.command_and_args[1..];

        if look_for_builtin(bin_name) {
            let is_last = i == command.len() - 1;
            previous_output = run_builtin_with_args(bin_path, args, previous_output, is_last);
        }
        else if let Some(bin_path) = search_path_for_bin(bin_name) {
            let is_last = i == command.len() - 1;
            previous_output = run_bin_with_args(bin_path, args, previous_output, is_last);
        } else {
            eprintln!("Binary not found: {}", bin_name);
            return None;
        }
    }

    println!("done executing commands");
    Some(0)
}

fn look_for_builtin(builtin_name: &str) -> bool {

    match builtin_name {
        "cd" => {true},
        "type" => {true},
        "ls" => {true},
        "cat" => {true},
        _ => false,
    }
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

fn run_bin_with_args(
    bin_path: PathBuf,
    args: &[String],
    input: Option<std::process::ChildStdout>,
    is_last: bool,
) -> Option<std::process::ChildStdout> {
    let mut cmd = Command::new(bin_path);
    cmd.args(args);

    if let Some(stdin) = input {
        cmd.stdin(Stdio::from(stdin));
    }

    if is_last {
        // Final command, we want to inherit output
        cmd.stdout(Stdio::inherit());
    } else {
        cmd.stdout(Stdio::piped());
    }

    match cmd.spawn() {
        Ok(mut child) => {
            if is_last {
                let status = child.wait().expect("failed to wait on child");
                println!("Process exited with: {}", status);
                None
            } else {
                child.stdout.take()
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            None
        }
    }
}

fn run_builtin_with_args(
    bin_path: PathBuf,
    args: &[String],
    input: Option<std::process::ChildStdout>,
    is_last: bool,
) -> Option<std::process::ChildStdout> {

    None
}
