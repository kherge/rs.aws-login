//! A subcommand used to configure Docker to use the AWS Elastic Container Registry.

use crate::app::Application;
use crate::util::run::Run;
use carli::error::Context;
use carli::prelude::cmd::*;

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        let registry_uri = generate_registry_uri(context)?;
        let password = Run::new("aws")
            .with_aws_options(context)
            .arg("ecr")
            .arg("get-login-password")
            .output()
            .context(|| "Could not generate ECR password.".to_owned())?;

        Run::new("docker")
            .arg("login")
            .arg("--username")
            .arg("AWS")
            .arg("--password")
            .arg(&password)
            .arg(&registry_uri)
            .pass_through(context)
            .context(|| "Docker could not be configured to use the registry.".to_owned())?;

        Ok(())
    }
}

/// Generates the ECR registry URI using the active profile.
fn generate_registry_uri(context: &Application) -> Result<String> {
    let account_id = Run::new("aws")
        .with_aws_options(context)
        .arg("sts")
        .arg("get-caller-identity")
        .arg("--query")
        .arg("Account")
        .arg("--output")
        .arg("text")
        .output()
        .map(|output| output.trim().to_owned())
        .context(|| "Could not get account ID from AWS CLI.".to_owned())?;

    let region = match context.region() {
        Some(region) => region.to_owned(),
        None => {
            let output = Run::new("aws")
                .with_aws_options(context)
                .arg("configure")
                .arg("get")
                .arg("region")
                .output()
                .map(|output| output.trim().to_owned())
                .context(|| "Could not get default region from AWS CLI.".to_owned())?;

            if output.is_empty() {
                err!(1, "The region could not be determined.");
            }

            output
        }
    };

    Ok(format!("{}.dkr.ecr.{}.amazonaws.com", account_id, region))
}
