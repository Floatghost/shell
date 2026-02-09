use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
mod app;
mod command;
mod completion;
mod config;
mod editor;
mod execute;
mod find;
mod highlight;
mod input;
mod parser;
mod prompt_render;
mod render;
mod userinput;

use crate::{
    app::State,
    command::{BuiltIn, Command, Runner, malloc_stress_process},
    config::{Color, ShellConfig},
    editor::Editor,
    prompt_render::construct_prompt,
    render::RenderEngine,
};

pub static CTRLC_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main() {
    ctrlc::set_handler(move || {
        let prev = CTRLC_COUNT.fetch_add(1, Ordering::SeqCst);

        if prev >= 1 {
            std::process::exit(0);
        }

        println!();
    })
    .expect("failed to set ctrl-c handler");

    if std::env::args().any(|a| a == "--malloc-stress") {
        malloc_stress_process(Duration::from_secs(20));
        return;
    }

    let mut state = State::new();
    let mut config = ShellConfig::new().unwrap();

    enable_raw_mode().unwrap();
    let _ = ExitStrat;

    let mut return_code = None;

    let mut editor = Editor::new();
    let mut renderer = RenderEngine::new();

    let mut history: Vec<String> = Vec::new();

    loop {
        state.refresh();
        let prompt = construct_prompt(&state, &config);

        let userinput = editor
            .get_userinput(&mut renderer, &prompt, &history)
            .unwrap();

        history.push(userinput.clone());

        if userinput.trim().is_empty() {
            continue;
        }

        if let Some(command) = Command::new(&userinput) {
            if let Runner::InBuilt(com) = &command.runner
                && com == &BuiltIn::Exit
            {
                print!("\x1b[0m");
                println!("return {:?}", return_code);
                break;
            }

            disable_raw_mode().unwrap();
            return_code = command.exec(&mut state, &mut config);
            enable_raw_mode().unwrap();
        } else {
            println!(
                "{}invalid input \"{}\"\x1b[0m",
                Color::Red.to_ansi_fg(),
                userinput
            );
        }

        editor.clear();
    }
}

struct ExitStrat;

impl Drop for ExitStrat {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
