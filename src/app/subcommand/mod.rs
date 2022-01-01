//! Provides the application subcommands to be executed.

mod debug;
mod ecr;
mod eks;
mod profile;
mod sso;

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

    /// Configures kubectl to use a cluster in AWS EKS.
    ///
    /// This subcommand will prompt you to select one cluster out of any that are found in EKS
    /// for your active AWS CLI profile. Once a cluster is selected, kubectl's configuration
    /// will be updated to support accessing it.
    Eks(eks::Subcommand),

    /// Makes an AWS CLI profile the active profile.
    ///
    /// This subcommand will first check if the profile exists. If the profile does not exist but
    /// has a corresponding template, the AWS CLI profile will be created using the template. If
    /// the AWS CLI profile exists, or was created, it will be made an active AWS CLI profile.
    /// This activation removes the need to use the --profile option for every AWS CLI execution.
    #[structopt(name = "use")]
    Profile(profile::Subcommand),

    /// Logs into an AWS account using SSO.
    ///
    /// This subcommand will attempt to log into the AWS account configured for the active AWS CLI
    /// profile, or prompt you to configure the active profile for SSO authentication. If the AWS
    /// CLI profile requires configuration, it will be a one time event. Future attempts will use
    /// the settings already provided.
    Sso(sso::Subcommand),
}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        match self {
            Self::Ecr(cmd) => cmd.execute(context),
            Self::Eks(cmd) => cmd.execute(context),
            Self::Profile(cmd) => cmd.execute(context),
            Self::Sso(cmd) => cmd.execute(context),

            #[cfg(debug_assertions)]
            Self::Debug(cmd) => cmd.execute(context),
        }
    }
}
