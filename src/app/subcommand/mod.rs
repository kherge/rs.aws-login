//! Supporting functions, types, and traits for implementing and creating subcommands.

mod ecr;
mod eks;

#[cfg(test)]
use std::fmt;
use std::{io, process, result};
use structopt::StructOpt;

/// A trait for objects which manage application settings.
pub trait Context {
    /// Returns the name of the profile.
    fn profile(&self) -> Option<&str>;

    /// Returns the name of the region.
    fn region(&self) -> Option<&str>;
}

/// An error that results from executing a subcommand that has failed.
#[derive(Debug)]
pub struct Error {
    /// The error message.
    message: Option<String>,

    /// The exit status code.
    status: i32,
}

impl Error {
    /// Exits the process using the error.
    pub fn exit(&self, error: &mut impl io::Write) -> ! {
        if let Some(message) = self.message.as_ref() {
            writeln!(error, "{}", message).unwrap();
        }

        process::exit(self.status);
    }

    /// Creates a new instance.
    pub fn new(status: i32, message: Option<String>) -> Self {
        Self { message, status }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            message: Some(format!("{}", error)),
            status: error.raw_os_error().unwrap_or(1),
        }
    }
}

/// A macro to return an error with a status, message, and/or formatted message.
#[macro_export]
macro_rules! err {
    ($status:expr) => {
        return Err(crate::app::subcommand::Error::new($status, None))
    };
    ($status:expr, $message:expr) => {
        return Err(crate::app::subcommand::Error::new($status, Some($message.into())))
    };
    ($status:expr, $message:tt, $($args:tt)*) => {
        return Err(crate::app::subcommand::Error::new($status, Some(format!($message, $($args)*))))
    };
}

/// A trait for objects which are executed as application subcommands.
pub trait Execute {
    /// Executes the subcommand using the given context and output streams.
    fn execute(
        &self,
        context: &impl Context,
        error: &mut impl io::Write,
        output: &mut impl io::Write,
    ) -> Result<()>;
}

/// A specialized Result type for subcommand operations.
pub type Result<T> = result::Result<T, Error>;

/// The available subcommands.
#[derive(StructOpt)]
pub enum Subcommand {
    /// Configures the Docker client to use ECR.
    Ecr(ecr::Subcommand),
    /// Configures the Kubernetes client to use EKS.
    Eks(eks::Subcommand),
}

impl Execute for Subcommand {
    fn execute(
        &self,
        context: &impl Context,
        error: &mut impl io::Write,
        output: &mut impl io::Write,
    ) -> Result<()> {
        match self {
            Self::Ecr(cmd) => cmd.execute(context, error, output),
            Self::Eks(cmd) => cmd.execute(context, error, output),
        }
    }
}

/// An application context used for testing.
#[cfg(test)]
#[derive(Default)]
pub struct TestContext {
    /// The name of the profile.
    pub profile: Option<String>,

    /// The name of the region.
    pub region: Option<String>,
}

#[cfg(test)]
impl Context for TestContext {
    fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}

/// A buffered output stream used for testing.
#[cfg(test)]
pub struct TestStream {
    /// The buffer containing the written contents.
    buffer: Vec<u8>,
}

#[cfg(test)]
impl fmt::Debug for TestStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8_lossy(&self.buffer).replace('\n', "\\n")
        )
    }
}

#[cfg(test)]
impl Default for TestStream {
    fn default() -> Self {
        Self { buffer: Vec::new() }
    }
}

#[cfg(test)]
impl io::Write for TestStream {
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }
}

#[cfg(test)]
impl PartialEq<&str> for TestStream {
    fn eq(&self, other: &&str) -> bool {
        String::from_utf8_lossy(&self.buffer) == *other
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// A test subcommand that will fail.
    struct ErrorTest {}

    impl Execute for ErrorTest {
        fn execute(
            &self,
            _: &impl Context,
            _: &mut impl io::Write,
            _: &mut impl io::Write,
        ) -> Result<()> {
            err!(123, "Oops!");
        }
    }

    /// A test subcommand that will succeed.
    struct SuccessTest {}

    impl Execute for SuccessTest {
        fn execute(
            &self,
            context: &impl Context,
            error: &mut impl io::Write,
            output: &mut impl io::Write,
        ) -> Result<()> {
            writeln!(error, "This is a test subcommand.")?;
            writeln!(output, "Profile: {:?}", context.profile())?;
            writeln!(output, " Region: {:?}", context.region())?;

            Ok(())
        }
    }

    #[test]
    fn error_macro_status_only() {
        let test = || -> Result<()> { err!(1) };
        let error = test().unwrap_err();

        assert_eq!(error.message, None);
        assert_eq!(error.status, 1);
    }

    #[test]
    fn error_macro_message() {
        let test = || -> Result<()> { err!(2, "The message.") };
        let error = test().unwrap_err();

        assert_eq!(error.message, Some("The message.".into()));
        assert_eq!(error.status, 2);
    }

    #[test]
    fn error_macro_formatted_message() {
        let test = || -> Result<()> { err!(3, "The {}.", "message") };
        let error = test().unwrap_err();

        assert_eq!(error.message, Some("The message.".into()));
        assert_eq!(error.status, 3);
    }

    #[test]
    fn execute_failing_subcommand() {
        let context = TestContext::default();
        let mut error = TestStream::default();
        let mut output = TestStream::default();

        let result = ErrorTest {}
            .execute(&context, &mut error, &mut output)
            .unwrap_err();

        assert_eq!(result.message, Some("Oops!".into()));
        assert_eq!(result.status, 123);
    }

    #[test]
    fn execute_successful_subcommand() {
        let context = TestContext::default();
        let mut error = TestStream::default();
        let mut output = TestStream::default();

        SuccessTest {}
            .execute(&context, &mut error, &mut output)
            .unwrap();

        assert_eq!(error, "This is a test subcommand.\n");
        assert_eq!(output, "Profile: None\n Region: None\n");
    }
}
