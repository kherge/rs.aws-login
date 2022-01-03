//! A subcommand used to download profile templates from a URL.

use crate::app::{self, profile, ErrorContext};
use crate::util::term;
use std::{fmt, str};

/// The options the user has to resolve multiple profile templates files.
enum Resolve {
    /// If a local templates file exists, do nothing.
    Cancel,

    /// Merge the two files, with remote templates replacing local ones of the same name.
    Merge,

    /// Remove the local templates and replace them with the remote ones.
    Replace,
}

impl fmt::Display for Resolve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cancel => "Cancel the download.",
                Self::Merge => "Merge with the existing templates.",
                Self::Replace => "Replace the existing templates.",
            }
        )
    }
}

impl str::FromStr for Resolve {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cancel" => Ok(Self::Cancel),
            "merge" => Ok(Self::Merge),
            "replace" => Ok(Self::Replace),
            _ => Err(s.to_owned()),
        }
    }
}

/// The options for the subcommand.
#[derive(structopt::StructOpt)]
pub struct Subcommand {
    /// How to handle the existing local profile templates.
    ///
    /// If a local profile templates file already exists, the application needs to know how to
    /// handle it. The options are to: cancel, merge, or replace. If "cancel" is chosen, then
    /// nothing is done and the local is preserved as is. If "merge" is chosen, the templates
    /// in the local file are preserved unless a remote one of the same name is found. If
    /// "replace" is chosen, all of the local profiles will be removed before being replaced
    /// by the remote templates.
    #[structopt(short, long)]
    resolve: Option<Resolve>,

    /// The URL to download the profile templates from.
    url: String,
}

impl app::Execute for Subcommand {
    fn execute(&self, _: &mut impl app::Context) -> app::Result<()> {
        let json = match reqwest::blocking::get(&self.url) {
            Ok(response) => match response.text() {
                Ok(text) => text,
                Err(error) => {
                    return Err(app::Error::new(1)
                        .with_message(format!("{}", error))
                        .with_context("The download response could not be read.".to_string()))
                }
            },
            Err(error) => {
                return Err(app::Error::new(1)
                    .with_message(format!("{}", error))
                    .with_context("The templates could not be downloaded.".to_string()))
            }
        };

        let remote = profile::parse_templates(json.as_bytes())
            .with_context(|| "Could not parse the downloaded templates.".to_owned())?;

        let mut templates = profile::get_templates()?;

        if templates.is_empty() {
            profile::set_templates(&remote)
                .with_context(|| "Could not save the downloaded templates.".to_owned())?;
        } else {
            let resolve = match &self.resolve {
                Some(resolve) => resolve,
                None => {
                    let prompt = "What would you like to do with the existing templates?";
                    let choices = &[Resolve::Cancel, Resolve::Merge, Resolve::Replace];

                    term::select(prompt, choices)?
                }
            };

            match &resolve {
                Resolve::Merge => {
                    for (name, template) in remote {
                        templates.insert(name, template);
                    }

                    profile::set_templates(&templates)
                        .with_context(|| "Could not update local templates.".to_owned())?;
                }
                Resolve::Replace => profile::set_templates(&remote)
                    .with_context(|| "Could not save the downloaded templates.".to_owned())?,
                _ => {
                    // Do nothing.
                }
            }
        }

        Ok(())
    }
}
