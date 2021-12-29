//! A command line utility to simplify logging into AWS accounts and services.

use std::io;
use structopt::StructOpt;

mod app;
mod util;

/// Initializes the application and processes the command line arguments.
fn main() {
    let app = app::Application::from_args_safe();
    let mut stderr = io::stderr();
    let mut stdout = io::stdout();

    // Run the application.
    if let Ok(app) = &app {
        if let Err(error) = app.execute(&mut stderr, &mut stdout) {
            error.exit(&mut stderr);
        }
    }

    // Exit evaluation loop if we're in one.
    if let Some(shell) = util::SHELL.lock().unwrap().as_mut() {
        shell.exit();
    }

    // Finally, exit with clap errors if available.
    if let Err(error) = &app {
        error.exit();
    }
}
