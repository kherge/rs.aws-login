//! A subcommand used for testing the application from the command line.

use crate::{app::Application, errorln, outputln};
use carli::error::Error;
use carli::prelude::cmd::*;

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// Causes the command to produce an error.
    #[clap(short, long)]
    pub(crate) error: bool,
}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        if self.error {
            errorln!(context, "Producing an error response.\n")?;

            let error = Error::new(123)
                .message("The --error option was used.".to_owned())
                .context("The subcommand could not complete successfully.".to_owned());

            return Err(error);
        }

        outputln!(context, "Producing a successful response.")?;

        Ok(())
    }
}
