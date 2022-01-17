//! Provides the primary command line application interface.

use crate::app::subcommand::Subcommand;
use carli::io::Stream;
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
