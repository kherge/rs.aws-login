//! A subcommand used to generate a token for accessing RDS Proxy using IAM.

use crate::app::ErrorContext;
use crate::util::{run, term};
use crate::{app, err, errorln};
use std::fmt;

/// Represents an RDS Proxy that is available.
struct Proxy {
    /// The host name for the endpoint of the proxy.
    endpoint: String,

    /// The database engine family.
    engine: String,

    /// The name of the proxy.
    name: String,

    /// The flag used to indicate if TLS is required.
    require_tls: bool,
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// The options for the subcommand.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// The database port number.
    #[clap(short, long)]
    port: Option<String>,

    /// The database username.
    username: String,
}

impl app::Execute for Subcommand {
    fn execute(&self, context: &mut impl app::Context) -> app::Result<()> {
        let proxies = get_proxies(context)?;
        let proxy = term::select("Please select an RDS Proxy:", &proxies)?;

        if proxy.engine != "POSTGRESQL" && self.port.is_none() {
            err!(
                1,
                "The database server port number is required for {} engines.",
                proxy.engine
            );
        }

        if proxy.require_tls {
            errorln!(
                context,
                "Warning: This connection requires TLS to be used.\n"
            )?;
        }

        run::Run::new("aws")
            .with_aws_options(context)
            .arg("rds")
            .arg("generate-db-auth-token")
            .arg("--hostname")
            .arg(&proxy.endpoint)
            .arg("--port")
            .arg(&self.port.as_deref().unwrap_or("5432"))
            .arg("--username")
            .arg(&self.username)
            .pass_through(context)?;

        Ok(())
    }
}

/// Retrieves a list of the available RDS Proxies.
fn get_proxies(context: &impl app::Context) -> app::Result<Vec<Proxy>> {
    let pairs = run::Run::new("aws")
        .with_aws_options(context)
        .arg("rds")
        .arg("describe-db-proxies")
        .arg("--query")
        .arg("DBProxies[].[DBProxyName,Endpoint,EngineFamily,RequireTLS, Status]")
        .arg("--output")
        .arg("text")
        .output()
        .map(|output| output.trim().to_owned())
        .with_context(|| "Could not get RDS Proxy host names from AWS CLI.".to_owned())?
        .split('\n')
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    let mut host_names = Vec::new();

    for pair in pairs {
        let mut parts = pair
            .split('\t')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        let (status, require_tls, engine, endpoint, name) = (
            parts.remove(4),
            parts.remove(3),
            parts.remove(2),
            parts.remove(1),
            parts.remove(0),
        );

        if status == "available" {
            let proxy = Proxy {
                require_tls: require_tls
                    .to_lowercase()
                    .parse::<bool>()
                    .expect("The RequireTLS field from the AWS CLI is not a boolean value."),
                endpoint,
                engine,
                name,
            };

            host_names.push(proxy);
        }
    }

    Ok(host_names)
}
