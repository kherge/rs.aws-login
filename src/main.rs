//! A command line utility to simplify logging into AWS accounts and services.

mod app;
mod util;

use structopt::StructOpt;

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
