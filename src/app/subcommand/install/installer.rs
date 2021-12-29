//! Manages the installation of the script used to integrate the binary into the shell.

use crate::app::subcommand;
use crate::{debug, err};
use std::io::Write;
use std::{fs, path, str};

/// The name of the integration script.
const SCRIPT_NAME: &str = ".aws-login.sh";

/// The shell script template.
const TEMPLATE: &str = r#"#!/bin/sh

# Get the path to the command.
export AWS_LOGIN=

if ! AWS_LOGIN="$(which aws-login)"; then
    echo "The aws-login command could not be found in PATH." >&2

    return 1
fi

###
# Integrates the AWS Login command with the shell.
#
# This function will be invoked instead of the command whenever you type
# `aws-login` and is responsible for evaluating any shell code it writes
# to a file.
#
# shellcheck disable=SC3033
# shellcheck disable=SC3043
##
{function}()
{
    # Create the shell script file.
    local SCRIPT=

    if SCRIPT="$(mktemp)"; then

        # Execute the real command.
        SHELL_PIPE="$SCRIPT" "$AWS_LOGIN" "$@"

        local STATUS=$?

        # Evaluate the shell script if it is not empty.
        if [ -s "$SCRIPT" ]; then
            eval "$(cat "$SCRIPT")"
        fi

        rm "$SCRIPT"

        return $STATUS
    fi

    return 1
}
"#;

/// Manages installation information for a specific shell.
pub struct Installer {
    /// The function name to use.
    function: String,

    /// The startup script injection template.
    inject: String,

    /// The path to the integration script.
    script: path::PathBuf,

    /// The path to the startup script.
    startup: path::PathBuf,
}

impl Installer {
    /// Creates an instance for a shell.
    pub fn for_shell(shell: &Shell) -> subcommand::Result<Installer> {
        let home = match home::home_dir() {
            Some(home) => home,
            None => err!(1, "The home directory path could not be determined."),
        };

        match shell {
            Shell::BASH => {
                debug!("We are running in BASH.");

                Ok(Installer {
                    function: "aws-login".to_owned(),
                    inject: ". {script}".to_owned(),
                    script: home.join(SCRIPT_NAME),
                    startup: home.join(".bashrc"),
                })
            }
            Shell::POSIX => {
                debug!("Assuming a POSIX compliant shell.");

                Ok(Installer {
                    function: "aws_login".to_owned(),
                    inject: "source {script}".to_owned(),
                    script: home.join(SCRIPT_NAME),
                    startup: home.join(".profile"),
                })
            }
            Shell::ZSH => {
                debug!("We are running in ZSH.");

                Ok(Installer {
                    function: "aws-login".to_owned(),
                    inject: "source {script}".to_owned(),
                    script: home.join(SCRIPT_NAME),
                    startup: home.join(".zshrc"),
                })
            }
        }
    }

    /// Installs the integration script.
    pub fn install(&self) -> subcommand::Result<()> {
        debug!("Beginning installation...");

        self.install_script()?;
        self.install_startup()?;

        Ok(())
    }

    /// Checks if the integration script is already installed.
    pub fn is_installed(&self) -> subcommand::Result<bool> {
        debug!("Checking if script is installed...");

        Ok(self.script.exists() && self.startup_contains(self.script.to_str().unwrap())?)
    }

    /// Creates the integration script.
    ///
    /// If the integration script already exist, it will not be touched.
    fn install_script(&self) -> subcommand::Result<()> {
        if !self.script.exists() {
            debug!("Creating the integration script.");

            let mut file = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&self.script)?;

            write!(file, "{}", TEMPLATE.replace("{function}", &self.function))?;
        }

        Ok(())
    }

    /// Inserts a line into the startup script to load the integration script.
    ///
    /// If the startup script already contains the line, it will not be touched.
    fn install_startup(&self) -> subcommand::Result<()> {
        let contents = if self.startup.exists() {
            fs::read_to_string(&self.startup)?
        } else {
            "".to_owned()
        };

        let script_path = self.script.to_str().unwrap();

        if !contents.contains(&script_path) {
            debug!("Adding line to startup script.");

            let mut file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.startup)?;

            writeln!(file, "{}", self.inject.replace("{script}", &script_path))?;
        }

        Ok(())
    }

    /// Checks if the startup script contains a string.
    fn startup_contains(&self, string: &str) -> subcommand::Result<bool> {
        Ok(fs::read_to_string(&self.startup)?.contains(string))
    }
}

/// The supported shells.
pub enum Shell {
    BASH,
    POSIX,
    ZSH,
}

impl str::FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bash" => Ok(Self::BASH),
            "posix" => Ok(Self::POSIX),
            "zsh" => Ok(Self::ZSH),
            _ => Err(format!("Unrecognized shell: {}", s)),
        }
    }
}
