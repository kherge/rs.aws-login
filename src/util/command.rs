//! Manages the execution of third-party command line applications.

use crate::app::subcommand;
use crate::error;
use std::{ffi, process};

/// Wraps the process builder to make it easier to use.
pub struct Run {
    /// The process builder.
    builder: process::Command,

    /// The name of the process.
    name: String,
}

impl Run {
    /// Adds an argument to the process builder.
    pub fn arg<S: AsRef<ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.builder.arg(arg);

        self
    }

    /// Creates a new instance for an application.
    pub fn new(program: &str) -> Self {
        Self {
            builder: process::Command::new(&program),
            name: program.to_owned(),
        }
    }

    /// Executes the process and captures its output.
    pub fn output(&mut self) -> subcommand::Result<String> {
        let process = self.builder.output()?;

        if process.status.success() {
            let output = match String::from_utf8(process.stdout) {
                Ok(string) => string,
                Err(_) => error!(1, "The output of, {}, is not valid UTF-8.", self.name),
            };

            Ok(output)
        } else {
            error!(
                process.status.code().unwrap_or(1),
                String::from_utf8(process.stderr)
                    .unwrap_or("The error output of, {}, is not valid UTF-8.".into())
            );
        }
    }

    /// Executes the process and passes through its output.
    pub fn pass_through(&mut self) -> subcommand::Result<()> {
        let process = self.builder.status()?;

        if process.success() {
            Ok(())
        } else {
            error!(process.code().unwrap_or(1))
        }
    }

    /// Pass through STDIN from our process.
    pub fn stdin(&mut self) -> &mut Self {
        self.builder.stdin(process::Stdio::inherit());

        self
    }
}

/// Creates a new process builder for the AWS CLI.
pub fn aws(context: &impl subcommand::Context) -> Run {
    let mut builder = Run::new("aws");

    if let Some(profile) = context.profile() {
        builder.arg("--profile").arg(profile);
    }

    if let Some(region) = context.region() {
        builder.arg("--region").arg(region);
    }

    builder
}
