//! Provides the application subcommands to be executed.

mod debug;
mod ecr;
mod eks;
mod profile;
mod pull;
mod shell;
mod sso;

use crate::app;

/// The subcommands available to the user.
#[derive(clap::Parser)]
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
    #[clap(name = "use")]
    Profile(profile::Subcommand),

    /// Downloads profile templates from a URL.
    ///
    /// This subcommand will download profile templates from a URL and store them in the local
    /// templates.json file (~/.config/aws-login/ or %APPDATA%\Roaming\AWS Login\). If a local
    /// templates file already exists, you will be asked to replace all of the templates or merge
    /// with the existing ones. If a merge is selected, the downloaded templates will replace any
    /// existing templates of the same name.
    Pull(pull::Subcommand),

    /// Integrates the application into the shell environment.
    ///
    /// This subcommand is capable of modifying the shell profile to inject code required to
    /// integrate the application into the environment. This integration allows the utility
    /// to make temporary modifications such as setting an environment variable. This is used in
    /// situations where it would simplify the use of the AWS CLI (such as setting AWS_PROFILE).
    Shell(shell::Subcommand),

    /// Logs into an AWS account using the AWS SSO portal.
    ///
    /// This subcommand will attempt to log into an account using the AWS SSO portal configured
    /// for the active AWS CLI profile, or prompt you to provide any missing settings before
    /// authentication can continue. The settings will be preserved the next time authentication
    /// is attempted.
    Sso(sso::Subcommand),
}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        match self {
            Self::Ecr(cmd) => cmd.execute(context),
            Self::Eks(cmd) => cmd.execute(context),
            Self::Profile(cmd) => cmd.execute(context),
            Self::Pull(cmd) => cmd.execute(context),
            Self::Shell(cmd) => cmd.execute(context),
            Self::Sso(cmd) => cmd.execute(context),

            #[cfg(debug_assertions)]
            Self::Debug(cmd) => cmd.execute(context),
        }
    }
}
