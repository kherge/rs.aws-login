//! A subcommand used to used to integrate the application with the user's shell.

use crate::app::Application;
use crate::outputln;
use crate::util::shell;
use carli::error::{Context, Error};
use carli::prelude::cmd::*;
use std::str;

/// The actions supported by the subcommand.
enum Action {
    /// Generate the startup script initialization shell code.
    Init,

    /// Modify the shell profile to inject our initialization shell code.
    Install,
}

impl str::FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(Self::Init),
            "install" => Ok(Self::Install),
            _ => Err(s.to_owned()),
        }
    }
}

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// What the subcommand should do with the shell environment.
    ///
    /// The subcommand is capable of a couple of actions: install, init. The install action will
    /// modify the shell profile's startup script to integrate this application. The init action
    /// will generate the initialization shell code for the integration.
    action: Action,

    /// The path to the shell profile's startup script (e.g. ~/.bashrc).
    #[clap(short, long)]
    init: Option<String>,

    /// The name of the shell used to manage the environment (e.g. bash).
    ///
    /// The subcommand needs to know what shell environment it will be modifying in order to
    /// provide shell specific support for the integration (e.g. Bash vs PowerShell). Please
    /// open a ticket to request support for additional shells.
    ///
    /// The supported shells are: bash, fish, powershell, zsh
    #[clap(short, long)]
    shell: String,
}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        let env = shell::get_setup(&self.shell, self.init.as_deref())
            .ok_or_else(|| Error::new(1).message("The shell is not supported.".to_owned()))?;

        match &self.action {
            Action::Init => outputln!(context, "{}", env.generate_script())
                .map_err(Error::from)
                .context(|| "Could not write initialization script to output.".to_owned())?,
            Action::Install => {
                let installed = env.is_installed().context(|| {
                    "Could not check if the integration is already set up.".to_owned()
                })?;

                if installed {
                    outputln!(context, "The integration is already installed.")?;
                } else {
                    env.install()
                        .context(|| "Could not install integration script.".to_owned())?
                }
            }
        }

        Ok(())
    }
}
