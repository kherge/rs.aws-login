//! Manages the configuration settings and files for the application.

use std::{env, fs, path};

lazy_static::lazy_static! {
    /// The path to the AWS CLI configuration directory.
    pub static ref AWS_CONFIG_DIR: path::PathBuf = match home::home_dir() {
        Some(mut path) => {
            path.push(".aws");

            path
        },
        None => panic!("The home directory could not be determined."),
    };

    /// The absolute path to the application binary.
    pub static ref BIN_NAME: String = env::current_exe()
        .map(|s| s.to_string_lossy().to_string())
        .expect("Could not create a string for the application name.");

    /// The path to the application configuration directory.
    pub static ref CONFIG_DIR: path::PathBuf = {
        let path = match home::home_dir() {
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

        if !path.exists() {
            fs::create_dir_all(&path)
                .unwrap_or_else(|_| panic!("The configuration directory could not be created."));
        }

        path
    };
}
