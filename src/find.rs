use std::{collections::btree_map::Entry, path::PathBuf};

use crate::app::State;

#[derive(Debug, Clone)]
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

pub fn find_in_local(state: &State) -> std::io::Result<Vec<Found>> {
    let entrys = std::fs::read_dir(state.cwd.clone())?;

    for entry in entrys {}

    todo!();
}
