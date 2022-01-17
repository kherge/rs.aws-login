//! Provides the primary command line application interface.

use crate::app::subcommand::Subcommand;
use carli::prelude::app::*;
use std::{cell, io};

/// Manages the global command line options.
#[derive(clap::Parser)]
#[clap(about, version, author)]
pub struct Application {
    /// The error output stream.
    #[clap(skip = cell::RefCell::new(io::stderr().into()))]
    pub(crate) error: cell::RefCell<Stream>,

    /// The input stream.
    #[clap(skip = cell::RefCell::new(io::stdin().into()))]
    pub(crate) input: cell::RefCell<Stream>,

    /// The error output stream.
    #[clap(skip = cell::RefCell::new(io::stdout().into()))]
    pub(crate) output: cell::RefCell<Stream>,

    /// Overrides the active AWS CLI profile.
    #[clap(long, global = true)]
    pub(crate) profile: Option<String>,

    /// Overrides the default AWS region.
    #[clap(long, global = true)]
    pub(crate) region: Option<String>,

    /// The subcommand to execute.
    #[clap(subcommand)]
    pub(crate) subcommand: Subcommand,
}

impl Application {
    /// Returns the name of the AWS CLI profile.
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Returns the name of the AWS region.
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Creates a new test instance of the application.
    #[cfg(any(doc, test))]
    pub fn test() -> Self {
        Self {
            error: cell::RefCell::new(Vec::new().into()),
            input: cell::RefCell::new(Vec::new().into()),
            output: cell::RefCell::new(Vec::new().into()),
            profile: None,
            region: None,
            subcommand: Subcommand::Debug(crate::app::subcommand::debug::Subcommand {
                error: false,
            }),
        }
    }
}

impl Main for Application {
    fn subcommand(&self) -> &dyn carli::command::Execute<Self> {
        &self.subcommand
    }
}

impl Shared for Application {
    fn error(&self) -> cell::RefMut<Stream> {
        self.error.borrow_mut()
    }

    fn input(&self) -> cell::RefMut<Stream> {
        self.input.borrow_mut()
    }

    fn output(&self) -> cell::RefMut<Stream> {
        self.output.borrow_mut()
    }
}
