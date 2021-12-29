//! The command for integrating the binary into the shell.

mod installer;

use crate::app::subcommand;
use std::io;
use structopt::StructOpt;

/// The integrate subcommand options.
#[derive(StructOpt)]
pub struct Subcommand {
    /// The shell to integrate into.
    #[structopt(short, long)]
    shell: installer::Shell,
}

impl subcommand::Execute for Subcommand {
    fn execute(
        &self,
        _: &impl subcommand::Context,
        _: &mut impl io::Write,
        output: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        let installer = installer::Installer::for_shell(&self.shell)?;

        if installer.is_installed()? {
            writeln!(output, "aws-login is already integrated into your shell.")?;
        } else {
            installer.install()?;
        }

        Ok(())
    }
}
