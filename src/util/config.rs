//! Manages the configuration settings and files for the application.

use std::{env, path};

lazy_static::lazy_static! {
    /// The path to the AWS CLI configuration directory.
    pub static ref AWS_CONFIG_DIR: path::PathBuf = match home::home_dir() {
        Some(mut path) => {
            path.push(".aws");

            path
        },
        None => panic!("The home directory could not be determined."),
    };

    /// The name of the binary being executed, which should be this application.
    ///
    /// The processes that depend on this value will all assume that the binary is available in
    /// the user's `PATH`. The installation guide must include information on this requirement to
    /// prevent any confusion on their end.
    pub static ref BIN_NAME: String = env::current_exe()
        .map(|path| {
            path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .expect("Could not create a string for the application name.")
        })
        .expect("The name of this application could not be determined.");

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
