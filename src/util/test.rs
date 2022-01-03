//! Provides types and functions used for testing subcommands.

use crate::app::Context;
use crate::{errorln, outputln};
use std::{io, str, sync};

/// Manages the context for subcommands executed in testing.
///
/// ```
/// let context = TestContext::default();
/// ```
#[derive(Default)]
pub struct TestContext {
    /// The error output buffer.
    error: sync::Arc<sync::Mutex<Vec<u8>>>,

    /// The standard output buffer.
    output: sync::Arc<sync::Mutex<Vec<u8>>>,

    /// The AWS CLI profile name.
    profile: Option<String>,

    /// The AWS region name.
    region: Option<String>,
}

impl Context for TestContext {
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

impl TestContext {
    /// Returns the error output buffer as a string slice.
    ///
    /// ```
    /// let context = TestContext::default();
    ///
    /// write!(context.error(), "The error.").unwrap();
    ///
    /// assert_eq!(context.error_as_str(), "The error.");
    /// ```
    pub fn error_as_string(&self) -> String {
        let lock = self.error.clone();
        let value = lock.lock().unwrap();

        str::from_utf8(&value).unwrap().to_owned()
    }

    /// Returns the standard output buffer as a string slice.
    ///
    /// ```
    /// let context = TestContext::default();
    ///
    /// write!(context.output(), "The output.").unwrap();
    ///
    /// assert_eq!(context.output_as_str(), "The output.");
    /// ```
    pub fn output_as_string(&self) -> String {
        let lock = self.output.clone();
        let value = lock.lock().unwrap();

        str::from_utf8(&value).unwrap().to_owned()
    }

    /// Sets the profile option while consuming self.
    ///
    /// ```
    /// let context = TestContext::default().with_profile("example".to_owned());
    /// ```
    pub fn with_profile(mut self, profile: String) -> Self {
        self.profile = Some(profile);

        self
    }

    /// Sets the region option while consuming self.
    ///
    /// ```
    /// let context = TestContext::default().with_region("us-east-1".to_owned());
    /// ```
    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_profile() {
        let context = TestContext {
            profile: Some("test".to_owned()),
            ..TestContext::default()
        };

        assert_eq!(context.profile(), Some("test"));
    }

    #[test]
    fn get_region() {
        let context = TestContext {
            region: Some("test".to_owned()),
            ..TestContext::default()
        };

        assert_eq!(context.region(), Some("test"));
    }

    #[test]
    fn read_write_error() {
        let mut context = TestContext::default();

        errorln!(context, "A test message.").unwrap();

        assert!(context.error_as_string().contains("A test message."));
    }

    #[test]
    fn read_write_output() {
        let mut context = TestContext::default();

        outputln!(context, "A test message.").unwrap();

        assert_eq!(context.output_as_string(), "A test message.\n");
    }

    #[test]
    fn with_profile_set() {
        let context = TestContext::default().with_profile("test".to_owned());

        assert_eq!(context.profile(), Some("test"));
    }

    #[test]
    fn with_region_set() {
        let context = TestContext::default().with_region("test".to_owned());

        assert_eq!(context.region(), Some("test"));
    }
}
