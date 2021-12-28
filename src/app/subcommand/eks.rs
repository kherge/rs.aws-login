//! The command for configuring `kubectl` to use EKS.

use crate::app::subcommand;
use crate::util;
use std::io;
use structopt::StructOpt;

/// The EKS subcommand options.
#[derive(StructOpt)]
pub struct Subcommand {}

impl subcommand::Execute for Subcommand {
    fn execute(
        &self,
        context: &impl subcommand::Context,
        _: &mut impl io::Write,
        _: &mut impl io::Write,
    ) -> subcommand::Result<()> {
        let clusters = get_clusters(context)?;
        let cluster = util::choose("Please choose an EKS cluster.\n", &clusters);

        util::aws(context)
            .arg("eks")
            .arg("update-kubeconfig")
            .arg("--name")
            .arg(cluster)
            .pass_through()?;

        Ok(())
    }
}

/// Retrieves the list of clusters available to the active profile.
fn get_clusters(context: &impl subcommand::Context) -> subcommand::Result<Vec<String>> {
    let clusters: Vec<String> = util::aws(context)
        .arg("eks")
        .arg("list-clusters")
        .arg("--query")
        .arg("clusters")
        .arg("--output")
        .arg("text")
        .output()?
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    Ok(clusters)
}
