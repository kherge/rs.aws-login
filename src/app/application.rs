//! Provides the primary command line application interface.

use crate::app::{subcommand, Execute};
use std::{io, sync};

/// Manages the global command line options.
#[derive(clap::Parser)]
#[clap(about, version, author)]
pub struct Application {
    /// Overrides the active AWS CLI profile.
    #[clap(long, global = true)]
    profile: Option<String>,

    /// Overrides the default AWS region.
    #[clap(long, global = true)]
    region: Option<String>,

    /// The subcommand to execute.
    #[clap(subcommand)]
    subcommand: subcommand::Subcommand,
}

impl Application {
    /// Creates a new context and executes the requested subcommand.
    ///
    /// ```
    /// use structopt::StructOpts;
    ///
    /// Application::from_args().execute();
    /// ```
    pub fn execute(&self) -> super::Result<()> {
        let mut context = ApplicationContext::new(self);

        self.subcommand.execute(&mut context)
    }
}

/// Manages the context in which subcommands are executed.
struct ApplicationContext {
    /// The error output stream.
    error: sync::Arc<sync::Mutex<io::Stderr>>,

    /// The standard output stream.
    output: sync::Arc<sync::Mutex<io::Stdout>>,

    /// The name of the AWS CLI profile.
    profile: Option<String>,

    /// The name of the AWS region.
    region: Option<String>,
}

impl ApplicationContext {
    /// Creates a new instance using the global options from the application.
    ///
    /// The `profile` and `region` options are cloned from the original application instance, but
    /// new instances of [`io::Stderr`] and [`io::Stdout`] are created for their respective error
    /// and standard output stream.
    ///
    /// ```
    /// use structopt::StructOpts;
    ///
    /// let application = Application::from_args();
    /// let mut context = ApplicationContext::new(&application);
    /// ```
    fn new(application: &Application) -> Self {
        Self {
            error: sync::Arc::new(sync::Mutex::new(io::stderr())),
            output: sync::Arc::new(sync::Mutex::new(io::stdout())),
            profile: application.profile.clone(),
            region: application.region.clone(),
        }
    }
}

impl super::Context for ApplicationContext {
    fn error(&mut self) -> sync::Arc<sync::Mutex<dyn io::Write>> {
        self.error.clone()
    }

    fn output(&mut self) -> sync::Arc<sync::Mutex<dyn io::Write>> {
        self.output.clone()
    }

    fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}
