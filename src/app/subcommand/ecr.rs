//! The command for configuring Docker to use ECR.

use crate::app::subcommand;
use crate::{err, util};
use std::io;
use structopt::StructOpt;

/// The ECR subcommand options.
#[derive(StructOpt)]
pub struct Subcommand {}

impl subcommand::Execute for Subcommand {
    fn execute(
        &self,
        context: &impl subcommand::Context,
        _: &mut impl io::Write,
        _: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        let registry_uri = generate_registry_uri(context)?;

        let password = util::aws(context)
            .arg("ecr")
            .arg("get-login-password")
            .output()?;

        util::Run::new("docker")
            .arg("login")
            .arg("--username")
            .arg("AWS")
            .arg("--password")
            .arg(password)
            .arg(registry_uri)
            .pass_through()?;

        Ok(())
    }
}

/// Generates the ECR registry URI using the active profile.
fn generate_registry_uri(context: &impl subcommand::Context) -> subcommand::Result<String> {
    let account_id = util::aws(context)
        .arg("sts")
        .arg("get-caller-identity")
        .arg("--query")
        .arg("Account")
        .arg("--output")
        .arg("text")
        .output()?
        .trim()
        .to_owned();

    let region = match context.region() {
        Some(region) => region.to_owned(),
        None => {
            let output = util::aws(context)
                .arg("configure")
                .arg("get")
                .arg("region")
                .output()?
                .trim()
                .to_owned();

            if output.is_empty() {
                err!(1, "The region could not be determined.");
            }

            output
        }
    };

    Ok(format!("{}.dkr.ecr.{}.amazonaws.com", account_id, region))
}
