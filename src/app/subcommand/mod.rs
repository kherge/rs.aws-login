//! Provides the application subcommands to be executed.

mod debug;
mod ecr;

use crate::app;

/// A command line utility to simplify logging into AWS accounts and services.
///
/// This utility serves as a wrapper around the AWS CLI to extend functionality that it already
/// provides. The goal is to merge together disparate but related commands into single subcommands
/// that are easier to remember and use. The utility also leverages templating for profiles that
/// may be shared with colleagues of the same organization, providing more consistent profile
/// naming conventions and configuration settings.
#[derive(structopt::StructOpt)]
pub enum Subcommand {
    /// Prints debugging messages for the application.
    #[cfg(debug_assertions)]
    Debug(debug::Subcommand),

    /// Configures Docker to use AWS ECR.
    ///
    /// This subcommand will generate the registry URI for the account in your active AWS CLI
    /// profile, generate a new ECR password, and use the Docker client to log into the AWS ECR
    /// service.
    Ecr(ecr::Subcommand),
}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        match self {
            Self::Ecr(cmd) => cmd.execute(context),

            #[cfg(debug_assertions)]
            Self::Debug(cmd) => cmd.execute(context),
        }
    }
}
