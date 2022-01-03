//! A command line utility to simplify logging into AWS accounts and services.
//!
//! This utility serves as a wrapper around the AWS CLI to extend functionality that it already
//! provides. The goal is to merge together disparate but related commands into single subcommands
//! that are easier to remember and use. The utility also leverages templating for profiles that
//! may be shared with colleagues of the same organization, providing more consistent profile
//! naming conventions and configuration settings.

mod app;
mod util;

use clap::Parser;

/// Primary entrypoint to the command line interface.
///
/// A new [`app::Application`] instance is created from any command line arguments that may have
/// been provided by the user. If an instance was successfully constructed, an attempt is made to
/// execute the requested subcommand. If the subcommand returns an error response, a clean up
/// operation is performed before the process exits with the error that was returned. If the
/// `app::Application` instance could not be created, a clean up operation is still performed
/// before exiting with the `clap::Error` returned by [`structopt::StructOpt::from_args_safe`].
fn main() {
    let mut app = app::Application::try_parse();

    if let Ok(app) = app.as_mut() {
        if let Err(error) = app.execute() {
            // NOTE Opening for clean up.

            error.exit()
        }
    }

    // NOTE Another opening for clean up.

    if let Err(error) = app.as_ref() {
        error.exit();
    }
}
