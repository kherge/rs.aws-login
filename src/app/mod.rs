//! Provides application and subcommand infrastructure.
//!
//! This module contains all of the infrastructure required to build an application capable of
//! invoking subcommands, including being able to test them. An example of this module supporting
//! testing is the use of the [`Context`] type, where we can specify our own error and standard
//! output streams for later testing.

mod application;
mod profile;
mod subcommand;

pub use application::Application;
