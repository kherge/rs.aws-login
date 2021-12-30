//! The command for downloading profile templates from a URL.

use crate::app::{profile, subcommand};
use crate::{err, util};
use std::{fs, io};
use structopt::StructOpt;

/// The pull subcommand options.
#[derive(StructOpt)]
pub struct Subcommand {
    /// The URL to the profile templates file.
    url: String,
}

impl subcommand::Execute for Subcommand {
    fn execute(
        &self,
        _: &impl subcommand::Context,
        _: &mut impl io::Write,
        output: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        if profile::TEMPLATES_FILE.exists() {
            let answer = util::choose("Replacing existing profile templates file?", &["No", "Yes"]);

            if *answer == "No" {
                writeln!(output, "Aborting.")?;

                return Ok(());
            }
        }

        let mut response = match reqwest::blocking::get(&self.url) {
            Ok(response) => response,
            Err(error) => err!(1, "The file could not be downloaded: {}", error),
        };

        if !util::CONFIG_DIR.exists() {
            fs::create_dir_all(util::CONFIG_DIR.as_path())?;
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(profile::TEMPLATES_FILE.as_path())?;

        match response.copy_to(&mut file) {
            Ok(_) => {}
            Err(error) => err!(1, "The file could not be written: {}", error),
        }

        writeln!(output, "Profiles downloaded succesfully.")?;

        Ok(())
    }
}
