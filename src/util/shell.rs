//! Manages interaction with a POSIX compliant shell.

use crate::app::subcommand;
use std::{env, fmt, fs, io, sync};

/// The name of the pipe used to evaluate shell code.
const PIPE_NAME: &str = "SHELL_PIPE";

lazy_static::lazy_static! {
    /// The instance used to evaluate shell code.
    pub static ref SHELL: sync::Mutex<Option<Shell>> = sync::Mutex::new(match env::var(PIPE_NAME) {
        Ok(name) => Some(Shell::from(fs::OpenOptions::new()
            .append(true)
            .open(name)
            .expect(&format!("{}: could not be opened", PIPE_NAME)))),
        Err(_) => None,
    });
}

/// Manages writing shell code to a stream.
pub struct Shell {
    /// The stream.
    stream: Box<dyn io::Write + Send>,
}

impl Shell {
    /// Exits the evaluation loop, if we are in one.
    pub fn exit(&mut self) {
        let _ = self.eval("break 2> /dev/null");
    }

    /// Exports the value for an environment variable.
    pub fn export<D: fmt::Display>(&mut self, name: D, value: D) -> subcommand::Result<()> {
        self.eval(format!("export {}=\"{}\"", name, value))
    }

    /// Writes shell code to the stream for evaluation.
    pub fn eval<D: fmt::Display>(&mut self, code: D) -> subcommand::Result<()> {
        writeln!(self.stream, "{}", code)?;

        Ok(())
    }
}

impl From<fs::File> for Shell {
    fn from(file: fs::File) -> Self {
        let stream = Box::new(file) as Box<dyn io::Write + Send>;

        Self { stream }
    }
}
