//! Manages the configuration settings and files for the application.

use std::path;

lazy_static::lazy_static! {
    /// The path to the application configuration directory.
    pub static ref CONFIG_DIR: path::PathBuf = match home::home_dir() {
        Some(mut path) => {
            if cfg!(windows) {
                path.push("AppData");
                path.push("Roaming");
                path.push("AWS Login");
            } else {
                path.push(".config");
                path.push("aws-login");
            }

            path
        },
        None => panic!("The home directory could not be determined."),
    };
}
