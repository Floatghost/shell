use std::path::PathBuf;

use crate::app::State;

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum Found {
    Path(PathBuf),
    EnvPath(PathBuf),
}

impl Found {
    pub fn file_name(&self) -> String {
        match self {
            Found::Path(p) => p.file_name().unwrap().to_str().unwrap().to_string(),
            Found::EnvPath(envp) => envp.file_name().unwrap().to_str().unwrap().to_string(),
        }
    }
}

pub fn complete_command(prefix: &str) -> Vec<Found> {
    let mut out = Vec::new();
    let paths = std::env::var("PATH").unwrap_or_default();

    let pathext = std::env::var("PATHEXT").unwrap_or_default();
    let allowed_exec: Vec<String> = pathext
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_start_matches('.').to_ascii_lowercase())
        .collect();

    for p in paths.split(';') {
        if let Ok(entries) = std::fs::read_dir(p) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();

                if !name
                    .to_ascii_lowercase()
                    .starts_with(&prefix.to_ascii_lowercase())
                {
                    continue;
                }

                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase());

                let is_exec = match ext {
                    Some(ref e) => allowed_exec.contains(e),
                    None => false,
                };

                if is_exec {
                    out.push(Found::EnvPath(entry.path()));
                }
            }
        }
    }

    out
}

pub fn complete_path(prefix: &str) -> Vec<Found> {
    // convert all "\" to "/"
    let clean_prefix: String = prefix.chars().map(|c| if c == '\\' {'/'} else {c}).collect();

    let path = if clean_prefix.to_lowercase().starts_with("c:") {
        // for every new / extend the path if this fails but we arent in the last / jet discard
        let mut potential_path = PathBuf::from("C:\\");
        let parts: Vec<&str> = clean_prefix.split("/").skip(1).collect(); // skip C:\

        for part in parts {
            // work here
        }
    } else {

    };

    // check if starts with an drive letter and ":"
    //
    // otherwise check in local dir

    // create path to the dir

    // loop through the dir

// todo get drive letters
    println!("complete path");
    let path = if prefix.to_lowercase().starts_with("c:") {
        println!("drive");
    } else if prefix.starts_with("./") || prefix.starts_with(".\\") {

    } else {

    }

    todo!()
}

#[allow(unused)]
pub fn find_in_local(state: &State) -> std::io::Result<Vec<Found>> {
    let entrys = std::fs::read_dir(state.cwd.clone())?;

    for entry in entrys {}

    todo!();
}
