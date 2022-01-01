//! Provides support for evaluating shell code.

mod bash;
mod zsh;

use crate::app;
use std::env;

/// The name of the environment variable used to specify the shell name.
///
/// This environment variable is expected to be defined when working with the shell environment.
/// The initialization script that is used to integrate the application is required to define the
/// variable using the same name that was used to install it.
const SHELL_NAME: &str = "AWS_LOGIN_SHELL";

/// Implemented by types that modify or interact with the current shell environment.
pub trait Environment {
    /// Sets the value of an environment variable.
    ///
    /// ```
    /// use crate::util::shell;
    ///
    /// let mut env = shell::get_env();
    ///
    /// env.set_var("EXAMPLE", "Hello, world!")?;
    /// ```
    fn set_var(&mut self, name: &str, value: &str) -> app::Result<()>;
}

/// Implemented by types that integrate the application into the current shell environment.
pub trait Setup {
    /// Generates the shell code used by a startup script to integrate the application.
    fn generate_script(&self) -> String;

    /// Modifies the profile's startup script to integration the application.
    fn install(&self) -> app::Result<()>;

    /// Checks if the integration script is already installed in the startup script.
    fn is_installed(&self) -> app::Result<bool>;
}

/// Returns the [`Environment`] implementation best suited for the current shell environment.
///
/// This function will use the `AWS_LOGIN_SHELL` environment variable to determine which shell
/// support module should be used to manage the environment. If the value does not correspond
/// to any supported shell, [`None`] is returned.
pub fn get_env() -> Option<Box<dyn Environment>> {
    match env::var(SHELL_NAME).as_deref() {
        Ok("bash") => Some(Box::new(bash::Environment::default())),
        Ok("zsh") => Some(Box::new(zsh::Environment::default())),
        _ => return None,
    }
}

/// Return the [`Setup`] implementation best suited for the specified shell.
///
/// This function will use the specified `shell` to determine which shell support module should
/// be used to configure the environment. If the specified shell does not correspond to any shell
/// that is supported, [`None`] is returned.
pub fn get_setup(shell: &str, profile: Option<&str>) -> Option<Box<dyn Setup>> {
    match shell {
        "bash" => Some(Box::new(bash::Setup::new(profile))),
        "zsh" => Some(Box::new(zsh::Setup::new(profile))),
        _ => return None,
    }
}
