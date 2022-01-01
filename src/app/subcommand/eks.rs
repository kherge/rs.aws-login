//! A subcommand used to configure `kubectl` to use AWS Elastic Kubernetes Service.

use crate::app::ErrorContext;
use crate::util::{run, term};
use crate::{app, err};

/// The options for the subcommand.
#[derive(structopt::StructOpt)]
pub struct Subcommand {
    /// The name of the desired cluster.
    cluster: Option<String>,
}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        let clusters = get_clusters(context)?;
        let cluster = match self.cluster.as_ref() {
            Some(cluster) => {
                if !clusters.contains(cluster) {
                    err!(1, "The specified cluster is not available.");
                }

                cluster
            }
            None => term::select("Please select an EKS cluster to setup:", &clusters)
                .with_context(|| "Unable to select an EKS cluster.".to_owned())?,
        };

        run::Run::new("aws")
            .with_aws_options(context)
            .arg("eks")
            .arg("update-kubeconfig")
            .arg("--name")
            .arg(cluster)
            .pass_through(context)
            .with_context(|| "Could not get the AWS CLI to configure kubectl.".to_owned())?;

        Ok(())
    }
}

/// Retrieves the list of clusters available in EKS for the active AWS CLI profile.
fn get_clusters(context: &impl app::Context) -> app::Result<Vec<String>> {
    let clusters = run::Run::new("aws")
        .with_aws_options(context)
        .arg("eks")
        .arg("list-clusters")
        .arg("--query")
        .arg("clusters")
        .arg("--output")
        .arg("text")
        .output()
        .with_context(|| {
            "The list of available EKS clusters could not be retrieved from the AWS CLI.".to_owned()
        })?
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();

    Ok(clusters)
}
