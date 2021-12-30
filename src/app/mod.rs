//! Provides the command line interface and subcommands to execute.

pub mod profile;
pub mod subcommand;

use std::io;
use structopt::StructOpt;
use subcommand::Execute;

/// A command line utility to simplify logging into AWS accounts and services.
#[derive(StructOpt)]
pub struct Application {
    /// Use a specific AWS CLI or template profile to use.
    #[structopt(long, global = true)]
    profile: Option<String>,

    /// Use a specific AWS region, overriding profile and environment settings.
    #[structopt(long, global = true)]
    region: Option<String>,

    /// AWS account or managed service to log into
    #[structopt(subcommand)]
    subcommand: subcommand::Subcommand,
}

impl Application {
    /// Executes the request subcommand.
    pub fn execute(
        &self,
        error: &mut impl io::Write,
        output: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        self.subcommand.execute(self, error, output)
    }
}

impl subcommand::Context for Application {
    fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}
