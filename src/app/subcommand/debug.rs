//! A subcommand used for testing the application from the command line.

use crate::app::Application;
use carli::prelude::cmd::*;
use carli::{error, errorln, outputln};

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// Causes the command to produce an error.
    #[clap(short, long)]
    error: bool,
}

#[cfg(test)]
impl Subcommand {
    /// Creates a new instance.
    pub fn new(error: bool) -> Self {
        Self { error }
    }
}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        if self.error {
            errorln!(context, "Producing an error response.\n")?;

            let error = error!(123, "The --error option was used.")
                .context("The subcommand could not complete successfully.".to_owned());

            return Err(error);
        }

        outputln!(context, "Producing a successful response.")?;

        Ok(())
    }
}
