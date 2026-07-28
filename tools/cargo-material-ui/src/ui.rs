use std::io::{self, IsTerminal};

use cliclack::ProgressBar;

pub struct Ui {
    animated: bool,
}

impl Ui {
    pub fn new(no_animations: bool) -> Self {
        let animated = !no_animations
            && io::stderr().is_terminal()
            && std::env::var_os("CI").is_none()
            && std::env::var_os("NO_COLOR").is_none()
            && std::env::var("TERM").is_ok_and(|term| term != "dumb");

        Self { animated }
    }

    pub fn is_interactive() -> bool {
        io::stdin().is_terminal() && io::stderr().is_terminal()
    }

    pub fn intro(&self, title: &str) {
        if self.animated {
            let _ = cliclack::intro(title);
        } else {
            eprintln!("{title}");
        }
    }

    pub fn spinner(&self, message: impl Into<String>) -> Spinner {
        let message = message.into();
        if self.animated {
            let progress = cliclack::spinner();
            progress.start(message);
            Spinner(Some(progress))
        } else {
            eprintln!("… {message}");
            Spinner(None)
        }
    }

    pub fn success(&self, message: &str) {
        if self.animated {
            let _ = cliclack::outro(message);
        } else {
            eprintln!("ok: {message}");
        }
    }
}

pub struct Spinner(Option<ProgressBar>);

impl Spinner {
    pub fn finish(self, message: impl Into<String>) {
        let message = message.into();
        if let Some(progress) = self.0 {
            progress.stop(message);
        } else {
            eprintln!("ok: {message}");
        }
    }
}
