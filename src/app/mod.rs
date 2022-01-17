//! Provides application and subcommands.
//!
//! This module contains the application type and all of the subcommands used in the application
//! for processing subcommands. The subcommands are each placed in their own respective module for
//! better organization. To create a new subcommand, we would simply need to create a new module
//! and register it with the [`subcommand::Subcommand`] enum.

mod profile;
mod subcommand;

use crate::app::subcommand::Subcommand;
use carli::prelude::app::*;
use std::{cell, io};

/// Manages the global command line options.
#[derive(clap::Parser)]
#[clap(about, version, author)]
pub struct Application {
    /// The error output stream.
    #[clap(skip = cell::RefCell::new(io::stderr().into()))]
    error: cell::RefCell<Stream>,

    /// The input stream.
    #[clap(skip = cell::RefCell::new(io::stdin().into()))]
    input: cell::RefCell<Stream>,

    /// The error output stream.
    #[clap(skip = cell::RefCell::new(io::stdout().into()))]
    output: cell::RefCell<Stream>,

    /// Overrides the active AWS CLI profile.
    #[clap(long, global = true)]
    profile: Option<String>,

    /// Overrides the default AWS region.
    #[clap(long, global = true)]
    region: Option<String>,

    /// The subcommand to execute.
    #[clap(subcommand)]
    subcommand: Subcommand,
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
    pub fn test(profile: Option<String>, region: Option<String>) -> Self {
        use subcommand::debug;

        Self {
            error: cell::RefCell::new(Vec::new().into()),
            input: cell::RefCell::new(Vec::new().into()),
            output: cell::RefCell::new(Vec::new().into()),
            profile,
            region,
            subcommand: Subcommand::Debug(debug::Subcommand::new(false)),
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
