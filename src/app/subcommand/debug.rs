//! A subcommand used for testing the application from the command line.

use crate::{app, errorln, outputln};

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// Causes the command to produce an error.
    #[clap(short, long)]
    error: bool,
}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        if self.error {
            errorln!(context, "Producing an error response.\n")?;

            let error = app::Error::new(123)
                .with_message("The --error option was used.".to_owned())
                .with_context("The subcommand could not complete successfully.".to_owned());

            return Err(error);
        }

        outputln!(context, "Producing a successful response.")?;

        Ok(())
    }
}
