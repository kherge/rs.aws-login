//! A command line utility to simplify logging into AWS accounts and services.

mod app;
mod util;

use structopt::StructOpt;

/// Primary entrypoint to the command line interface.
///
/// A new [`app::Application`] instance is created from any command line arguments that may have
/// been provided by the user. If an instance was successfully constructed, an attempt is made to
/// execute the requested subcommand. If the subcommand returns an error response, a clean up
/// operation is performed before the process exits with the error that was returned. If the
/// `app::Application` instance could not be created, a clean up operation is still performed
/// before exiting with the `clap::Error` returned by [`structopt::StructOpt::from_args_safe`].
fn main() {
    let mut app = app::Application::from_args_safe();

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
