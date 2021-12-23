//! A command line utility to simplify logging into AWS accounts and services.

use std::io;
use structopt::StructOpt;

mod app;
mod util;

/// Initializes the application and processes the command line arguments.
fn main() {
    let app = app::Application::from_args();
    let mut stderr = io::stderr();
    let mut stdout = io::stdout();

    if let Err(error) = app.execute(&mut stderr, &mut stdout) {
        error.exit(&mut stderr);
    }
}
