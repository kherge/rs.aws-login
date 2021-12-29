//! The command for configuring AWS CLI profiles.

use crate::app::{profile, subcommand};
use crate::{debug, err, util};
use std::io;
use structopt::StructOpt;

/// The Pick subcommand options.
#[derive(StructOpt)]
pub struct Subcommand {}

impl subcommand::Execute for Subcommand {
    fn execute(
        &self,
        context: &impl subcommand::Context,
        error: &mut impl io::Write,
        output: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        let profiles = profile::read_templates()?;
        let name = match context.profile() {
            Some(name) => name,
            None => {
                let mut profiles = profiles
                    .values()
                    .filter(|p| p.enabled())
                    .collect::<Vec<&profile::Profile>>();

                profiles.sort();

                util::choose("Please choose a profile:", &profiles).name()
            }
        };

        if !profile::exists_in_cli(context, name)? {
            debug!("Creating the AWS CLI profile...");

            let profile = match profiles.get(name) {
                Some(profile) => profile,
                None => err!(1, "A template for the profile, {}, does not exist.", name),
            };

            profile::install_in_cli(profile)?;
        }

        if let Some(shell) = util::SHELL.lock().unwrap().as_mut() {
            shell.export("AWS_PROFILE", name)?;
        } else {
            writeln!(error, "The application is not integrated into the shell.").unwrap();
            writeln!(error, "Please run the following shell code manually:\n").unwrap();
            writeln!(output, "export AWS_PROFILE=\"{}\"", name).unwrap();
        }

        Ok(())
    }
}
