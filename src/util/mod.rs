//! Provides miscellaneous utilities that are shared by subcommands and test suites.

pub mod config;
pub mod run;
pub mod shell;
pub mod term;

#[cfg(any(doc, test))]
pub mod test;
