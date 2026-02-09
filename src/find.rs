use std::path::{Path, PathBuf};

use crate::command::{BUILTIN_COMMANDS, BuiltIn};

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum Found {
    Path(PathBuf),
    EnvPath(PathBuf),
    BuiltIn(BuiltIn),
}

impl Found {
    pub fn file_name(&self) -> String {
        match self {
            Found::Path(p) => p.file_name().unwrap().to_str().unwrap().to_string(),
            Found::EnvPath(envp) => envp.file_name().unwrap().to_str().unwrap().to_string(),
            Found::BuiltIn(b) => b.to_string(),
        }
    }
}

pub fn resolve(cmd: &str) -> Option<Found> {
    if let Some(b) = BuiltIn::from(cmd) {
        return Some(Found::BuiltIn(b));
    }

    let pathext = std::env::var("PATHEXT").unwrap_or_default();
    let allowed_exec: Vec<String> = pathext
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_start_matches('.').to_ascii_lowercase())
        .collect();

    let check_file = |path: PathBuf| -> Option<PathBuf> {
        if path.is_file() {
            return Some(path);
        }
        for ext in &allowed_exec {
            let mut p_os = path.as_os_str().to_owned();
            p_os.push(".");
            p_os.push(ext);
            let p = PathBuf::from(p_os);

            if p.is_file() {
                return Some(p);
            }
        }
        None
    };

    if let Ok(current_dir) = std::env::current_dir() {
        let local = current_dir.join(cmd);
        if let Some(p) = check_file(local) {
            return Some(Found::Path(p));
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let full = dir.join(cmd);
            if let Some(p) = check_file(full) {
                return Some(Found::EnvPath(p));
            }
        }
    }

    None
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
    let mut clean = prefix.replace("/", "\\");

    if clean.len() == 2 && clean.ends_with(':') {
        clean.push('\\');
    }

    let (dir, partial) = if clean.ends_with('\\') {
        (clean.clone(), "")
    } else {
        match clean.rsplit_once('\\') {
            Some((d, p)) => (format!("{}\\", d), p),
            None => (".\\".into(), clean.as_str()),
        }
    };

    let dir_path = Path::new(&dir);

    let entries = match std::fs::read_dir(dir_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        if !name
            .to_ascii_lowercase()
            .starts_with(&partial.to_ascii_lowercase())
        {
            continue;
        }

        out.push(Found::Path(entry.path()));
    }

    out
}

pub fn complete_builtin(prefix: &str) -> Vec<Found> {
    let mut out = Vec::new();

    for (command, builtin) in BUILTIN_COMMANDS.into_iter() {
        if command.starts_with(prefix) {
            out.push(Found::BuiltIn(builtin.to_owned()));
        }
    }

    out
}
