//! The command for logging into an AWS account using SSO.

use crate::app::subcommand;
use crate::{debug, util};
use std::io;
use structopt::StructOpt;

/// The required profile configuration parameters for SSO.
const CONFIG_LIST: &[&str] = &[
    "sso_account_id",
    "sso_region",
    "sso_role_name",
    "sso_start_url",
];

/// The SSO subcommand options.
#[derive(StructOpt)]
pub struct Subcommand {}

impl subcommand::Execute for Subcommand {
    fn execute(
        &self,
        context: &impl subcommand::Context,
        _: &mut impl io::Write,
        _: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        if can_login(context)? {
            debug!("Can log directly in using SSO.");

            util::aws(context).arg("sso").arg("login").pass_through()?;
        } else {
            debug!("Need to finish SSO configuration first.");

            util::aws(context)
                .arg("configure")
                .arg("sso")
                .stdin()
                .pass_through()?;
        }

        Ok(())
    }
}

/// Checks if the active profile is fully configured for SSO.
fn can_login(context: &impl subcommand::Context) -> subcommand::Result<bool> {
    let mut has = 0;

    for key in CONFIG_LIST {
        match util::aws(context)
            .arg("configure")
            .arg("get")
            .arg(key)
            .output()
        {
            Ok(value) => {
                if !value.trim().is_empty() {
                    has += 1;
                }
            }
            Err(error) => {
                if error.status != 1 {
                    return Err(error);
                }
            }
        };
    }

    debug!("Of {} settings, {} had a value.", CONFIG_LIST.len(), has);

    Ok(has == CONFIG_LIST.len())
}
