//! Provides application and subcommand infrastructure.
//!
//! This module contains all of the infrastructure required to build an application capable of
//! invoking subcommands, including being able to test them. An example of this module supporting
//! testing is the use of the [`Context`] type, where we can specify our own error and standard
//! output streams for later testing.

mod application;
mod subcommand;

pub use application::Application;

use std::{fmt, io, process};

/// A trait for objects that manage the context a subcommand is executed in.
///
/// The trait is used over a struct in order to allow flexibility in how the context is
/// implemented. This allows one context to be created for standard operating procedure,
/// and another purely for testing.
pub trait Context {
    /// Returns the error output stream.
    fn error(&mut self) -> &mut dyn io::Write;

    /// Returns the standard output stream.
    fn output(&mut self) -> &mut dyn io::Write;

    /// Returns the name of the AWS CLI profile.
    fn profile(&self) -> Option<&str>;

    /// Returns the name of the AWS region.
    fn region(&self) -> Option<&str>;
}

/// An error that can be used to exit the process with a status code.
///
/// This struct allows subcommands and other code to capture an error and provide an appropriate
/// error code. In cases where the error is already displayed, it is unnecessary to also provide
/// the same error message here. In other cases, it may be necessary to add context the deeper
/// the error occurs.
#[derive(Debug)]
pub struct Error {
    /// The context message stack.
    ///
    /// The messages in this stack are display in reverse order before the error message.
    context: Vec<String>,

    /// The error message.
    message: Option<String>,

    /// The exit status code.
    status: i32,
}

impl Error {
    /// Exits the process with the error message (and context).
    ///
    /// ```
    /// fn failing() -> Result<()> {
    ///     return Err(Error::new(123).with_message("Nope!".to_owned()));
    /// }
    ///
    /// fn main() {
    ///     if let Err(error) = failing() {
    ///         error.exit();
    ///     }
    /// }
    /// ```
    pub fn exit(&self) -> ! {
        if self.context.is_empty() || self.message.is_some() {
            let _ = eprint!("{}", self);
        }

        process::exit(self.status);
    }

    /// Creates a new instance for an exit status code.
    ///
    /// ```
    /// let error = Error::new(123);
    /// ```
    pub fn new(status: i32) -> Self {
        Self {
            context: Vec::new(),
            message: None,
            status,
        }
    }

    /// Adds a message to the context stack while consuming self.
    ///
    /// A context message is useful if the error message itself is too vague to be helpful when it
    /// is displayed. For example, the error message may contain a vague [`io::Error`] message, but
    /// we can add some context to it such as "failed to open file: /path/to/file".
    ///
    /// ```
    /// fn deep_failure() -> Result<()> {
    ///     Err(Error::new(123, "Is a directory (os error 21)".to_owned()))
    /// }
    ///
    /// fn failure() -> Result<()> {
    ///     deep_failure().map(|error| {
    ///         error.with_context("The file, /path/to/file, could not be written.")
    ///     })
    /// }
    ///
    /// fn main() {
    ///     if let Err(error) = failure() {
    ///         error.exit();
    ///     }
    /// }
    /// ```
    ///
    /// The above example would yield the following error output and exit with a `123` status:
    ///
    /// ```text
    /// The file, /path/to/file, could not be written.
    ///   Is a directory (os error 21)
    /// ```
    pub fn with_context(mut self, message: String) -> Self {
        self.context.push(message);

        self
    }

    /// Sets or replaces the error message while consuming self.
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);

        self
    }
}

/// Supports converting [`io::Error`] to [`Error`].
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            context: Vec::new(),
            message: Some(format!("{}", error)),
            status: error.raw_os_error().unwrap_or(1),
        }
    }
}

/// Supports displaying the error message.
///
/// The error message that is displayed may be preceeded by context messages that were added. As
/// more context messages are displayed, the indentation level is increased in order to denote the
/// depth of the error.
///
/// ```text
/// Failed to generate a new profile.
///   Could not write to: /path/to/file
///     Is a directory (os error 21)
/// ```
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut indent = 0;

        // Print the context messages first.
        for message in self.context.iter().rev() {
            writeln!(f, "{}{}", " ".repeat(indent * 2), message)?;

            indent += 1;
        }

        // Then print the error message, if any.
        if let Some(message) = self.message.as_deref() {
            writeln!(f, "{}{}", " ".repeat(indent * 2), message)?;
        }

        Ok(())
    }
}

/// A trait for objects that can be executed as a subcommand.
pub trait Execute {
    /// Executes the subcommand with the given context.
    fn execute(&self, context: &mut impl Context) -> Result<()>;
}

/// A macro shortcut for returning an error.
///
/// There are multiple ways this macro can be used: using only a status code, using a status code
/// and an error message, or using a status code with a formatted message. Naturally, as it is with
/// all macros, this saves a lot repeating code.
///
/// ```
/// fn only_status() -> Result<()> {
///     err!(123);
/// }
///
/// fn with_message() -> Result<()> {
///     err!(123, "The error message.");
/// }
///
/// fn with_formatted_message() -> Result<()> {
///     err!(123, "The {} message.", "formatted error");
/// }
/// ```
#[macro_export]
macro_rules! err {
    ($status:expr) => {
        return Err(crate::app::Error::new($status))
    };
    ($status:expr, $message:expr) => {
        return Err(crate::app::Error::new($status).with_message($message.to_owned()))
    };
    ($status:expr, $message:tt, $($args:tt)*) => {
        return Err(crate::app::Error::new($status).with_message(format!($message, $($args)*)))
    };
}

/// A specialized [`Result`] for subcommands.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::test::*;

    #[test]
    fn error_status_only() {
        let error = Error::new(123);

        assert_eq!(error.status, 123);
        assert_eq!(format!("{}", error), "");
    }

    #[test]
    fn error_with_context() {
        let error = Error::new(123)
            .with_message("The message.".to_owned())
            .with_context("The parent context.".to_owned())
            .with_context("The parent parent context.".to_owned());

        assert_eq!(error.status, 123);
        assert_eq!(error.message, Some("The message.".to_owned()));
        assert_eq!(error.context[0], "The parent context.".to_owned());
        assert_eq!(error.context[1], "The parent parent context.".to_owned());
        assert_eq!(
            format!("{}", error),
            "The parent parent context.\n  The parent context.\n    The message.\n"
        );
    }

    #[test]
    fn error_with_message() {
        let error = Error::new(123).with_message("The message.".to_owned());

        assert_eq!(error.status, 123);
        assert_eq!(error.message, Some("The message.".to_owned()));
        assert_eq!(format!("{}", error), "The message.\n");
    }

    /// A test subcommand that will produce an error.
    struct ErrorSubcommand {}

    impl Execute for ErrorSubcommand {
        fn execute(&self, context: &mut impl Context) -> Result<()> {
            writeln!(context.error(), "Test error output.")?;

            err!(123, "The command failed.");
        }
    }

    /// A test subcommand that will succeed.
    struct SuccessSubcommand {}

    impl Execute for SuccessSubcommand {
        fn execute(&self, context: &mut impl Context) -> Result<()> {
            writeln!(context.output(), "Test standard output.")?;

            Ok(())
        }
    }

    #[test]
    fn erroring_subcommand() {
        let mut context = TestContext::default();
        let subcommand = ErrorSubcommand {};
        let error = subcommand.execute(&mut context).unwrap_err();

        assert_eq!(error.message, Some("The command failed.".to_owned()));
        assert_eq!(error.status, 123);
        assert_eq!(context.error_as_str(), "Test error output.\n");
    }

    #[test]
    fn successful_subcommand() {
        let mut context = TestContext::default();
        let subcommand = SuccessSubcommand {};
        let result = subcommand.execute(&mut context);

        assert!(result.is_ok());
        assert_eq!(context.output_as_str(), "Test standard output.\n");
    }
}
