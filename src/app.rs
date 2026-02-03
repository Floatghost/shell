use std::path::PathBuf;

pub struct State {
    pub username: String,
    pub cwd: PathBuf,
}

impl State {
    pub fn new() -> State {
        State {
            username: match std::env::var("USERNAME") {
                Ok(name) => name,
                Err(_) => "".into(),
            },
            cwd: std::env::current_dir().unwrap(),
        }
    }

    pub fn cd(&mut self, target: &str) -> std::io::Result<()> {
        std::env::set_current_dir(target)?;
        self.cwd = std::env::current_dir()?;
        Ok(())
    }
}
