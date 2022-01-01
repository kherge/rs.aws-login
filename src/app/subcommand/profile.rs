//! A subcommand used to create and/or select an AWS CLI profile.

use crate::app::{self, profile, ErrorContext};
use crate::err;
use crate::util::{run, term};

/// The options for the subcommand.
#[derive(structopt::StructOpt)]
pub struct Subcommand {}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        let existing = get_existing_profiles(context)?;
        let profiles = profile::get_profiles()?;
        let profile = match context.profile() {
            Some(profile) => profile.to_owned(),
            None => {
                let mut merged = profiles
                    .keys()
                    .map(|s| s.as_str())
                    .chain(existing.iter().map(|s| s.as_str()))
                    .collect::<Vec<&str>>();

                merged.sort();
                merged.dedup();

                if merged.len() == 0 {
                    err!(1, "There are no profiles available to choose from.");
                }

                term::select("Please select a profile to use:", &merged)?.to_string()
            }
        };

        if !existing.iter().any(|p| p == profile.as_str()) {
            if let Some(profile) = profiles.get(&profile) {
                create_profile(context, profile)?;
            } else {
                err!(1, "The profile, {}, does not exist.", profile);
            }
        }

        // TODO export AWS_PROFILE={}

        Ok(())
    }
}

/// Creates the AWS CLI profile.
fn create_profile(context: &mut impl app::Context, profile: &profile::Profile) -> app::Result<()> {
    for (key, value) in profile.settings() {
        run::Run::new("aws")
            .arg("--profile")
            .arg(profile.name())
            .arg("configure")
            .arg("set")
            .arg(key)
            .arg(value)
            .pass_through(context)
            .with_context(|| format!("Could not set the profile setting, {}.", key))?;
    }

    Ok(())
}

/// Returns a list of existing AWS CLI profiles.
fn get_existing_profiles(context: &impl app::Context) -> app::Result<Vec<String>> {
    let profiles = run::Run::new("aws")
        .with_aws_options(context)
        .arg("configure")
        .arg("list-profiles")
        .output()
        .with_context(|| "Could not get a list of existing AWS CLI profiles.".to_owned())?
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    Ok(profiles)
}
