//! A subcommand used to authenticate into an AWS account using SSO.

use crate::app::Application;
use crate::util::run;
use carli::prelude::cmd::*;

/// The profile configuration settings required for SSO.
const REQUIRED_SETTINGS: &[&str] = &[
    "sso_account_id",
    "sso_region",
    "sso_role_name",
    "sso_start_url",
];

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        if is_configured(context)? {
            run::Run::new("aws")
                .with_aws_options(context)
                .arg("sso")
                .arg("login")
                .pass_through(context)
                .context(|| "Could not log in via SSO.".to_owned())?;
        } else {
            run::Run::new("aws")
                .with_aws_options(context)
                .arg("configure")
                .arg("sso")
                .pass_through(context)
                .context(|| "Could not configure AWS CLI profile for SSO.".to_owned())?;
        }

        Ok(())
    }
}

/// Checks if the active profile is fully configured for SSO.
fn is_configured(context: &Application) -> Result<bool> {
    let mut has = 0;

    for key in REQUIRED_SETTINGS {
        if let Ok(value) = run::Run::new("aws")
            .with_aws_options(context)
            .arg("configure")
            .arg("get")
            .arg(key)
            .output()
        {
            if !value.trim().is_empty() {
                has += 1;
            }
        };
    }

    Ok(has == REQUIRED_SETTINGS.len())
}
