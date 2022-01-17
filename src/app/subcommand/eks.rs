//! A subcommand used to configure `kubectl` to use AWS Elastic Kubernetes Service.

use crate::app::Application;
use crate::util::run::Run;
use crate::util::term::select;
use carli::prelude::cmd::*;

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// The name of the desired cluster.
    cluster: Option<String>,
}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        let clusters = get_clusters(context)?;
        let cluster = match self.cluster.as_ref() {
            Some(cluster) => {
                if !clusters.contains(cluster) {
                    err!(1, "The specified cluster is not available.");
                }

                cluster
            }
            None => select("Please select an EKS cluster to setup:", &clusters)
                .context(|| "Unable to select an EKS cluster.".to_owned())?,
        };

        Run::new("aws")
            .with_aws_options(context)
            .arg("eks")
            .arg("update-kubeconfig")
            .arg("--name")
            .arg(cluster)
            .pass_through(context)
            .context(|| "Could not get the AWS CLI to configure kubectl.".to_owned())?;

        Ok(())
    }
}

/// Retrieves the list of clusters available in EKS for the active AWS CLI profile.
fn get_clusters(context: &Application) -> Result<Vec<String>> {
    let clusters = Run::new("aws")
        .with_aws_options(context)
        .arg("eks")
        .arg("list-clusters")
        .arg("--query")
        .arg("clusters")
        .arg("--output")
        .arg("text")
        .output()
        .context(|| {
            "The list of available EKS clusters could not be retrieved from the AWS CLI.".to_owned()
        })?
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    Ok(clusters)
}
