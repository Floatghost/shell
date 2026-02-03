use std::path::PathBuf;
use std::process::Command;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct GitInfo {
    pub branch: String,
    pub dirty: bool,
}

pub struct State {
    pub username: String,
    pub cwd: PathBuf,
    pub system: System,
    pub git: Option<GitInfo>,
}

impl State {
    pub fn new() -> State {
        State {
            username: match std::env::var("USERNAME") {
                Ok(name) => name,
                Err(_) => "user".into(),
            },
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            system: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
            git: None,
        }
    }

    pub fn cd(&mut self, target: &str) -> std::io::Result<()> {
        std::env::set_current_dir(target)?;
        self.cwd = std::env::current_dir()?;
        Ok(())
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();

        self.git = self.get_git_info();
    }

    fn get_git_info(&self) -> Option<GitInfo> {
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .arg("-b")
            .current_dir(&self.cwd)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8(output.stdout).ok()?;
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.is_empty() {
            return None;
        }

        let branch_line = lines[0];
        let branch = if branch_line.starts_with("## ") {
            let raw = &branch_line[3..];
            if let Some(idx) = raw.find("...") {
                raw[..idx].to_string()
            } else {
                raw.to_string()
            }
        } else {
            "HEAD".to_string()
        };

        let dirty = lines.len() > 1;

        Some(GitInfo { branch, dirty })
    }
}
