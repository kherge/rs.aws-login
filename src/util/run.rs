//! Provides a simplified and well integrated interface to [`Command`].

use crate::app::Application;
use carli::err;
use carli::error::{Error, Result};
use carli::io::Shared;
use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;
use std::sync::Mutex;
use tokio::io::AsyncReadExt;
use tokio::join;
use tokio::process::Command;
use tokio::runtime::Runtime;
use which::which;

lazy_static::lazy_static! {
    /// Caches the check performed for each program in `PATH`.
    static ref CHECK_CACHE: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
}

/// Simplifies the building of a new [`Command`] instance.
pub struct Run {
    /// The arguments used with the builder.
    #[cfg(debug_assertions)]
    arguments: Vec<String>,

    /// The process builder.
    builder: Command,

    /// The name of the program.
    program: String,
}

impl Run {
    /// Adds an argument to the process builder.
    ///
    /// ```
    /// let mut run = Run::new("my-app")
    ///     .arg("arg1")
    ///     .arg("arg2")
    ///     .arg("arg3");
    /// ```
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.builder.arg(arg);

        #[cfg(debug_assertions)]
        self.arguments.push(arg.to_owned());

        self
    }

    /// Returns the arguments added to the builder.
    ///
    /// ```
    /// let mut run = Run::new("my-app")
    ///     .arg("arg1")
    ///     .arg("arg2")
    ///     .arg("arg3");
    ///
    /// let args = run.get_args();
    /// ```
    #[cfg(test)]
    pub fn get_args(&self) -> &Vec<String> {
        &self.arguments
    }

    /// Creates a new instance for the specified command line application.
    ///
    /// ```
    /// let mut run = Run::new("my-app");
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            #[cfg(debug_assertions)]
            arguments: Vec::new(),
            builder: Command::new(name),
            program: name.to_owned(),
        }
    }

    /// Runs the command and returns its output.
    ///
    /// ```
    /// use crate::app;
    /// use crate::util::run;
    /// use structopt::StructOpts;
    ///
    /// let app = app::Application::from_args();
    /// let mut context = app::ApplicationContext::new(&app);
    ///
    /// let output = run::Run::new("aws")
    ///     .with_aws_options(&context)
    ///     .arg("eks")
    ///     .arg("list-clusters")
    ///     .output()?;
    ///
    /// println!("{}", output);
    /// ```
    ///
    /// If the command exits with a non-zero status, an [`Err`] for [`Result`] will be returned,
    /// with the error output being used as the message. It is recommended that context be added
    /// for these errors.
    pub fn output(&mut self) -> Result<String> {
        if !in_path(&self.program)? {
            err!(1, "The program, {}, could be found in PATH.", self.program);
        }

        Runtime::new()?.block_on(async {
            let output = self.builder.stdin(Stdio::inherit()).output().await?;

            if output.status.success() {
                let string = String::from_utf8_lossy(output.stdout.as_slice());

                Ok((*string).to_string())
            } else {
                let message = format!("{}", String::from_utf8_lossy(output.stderr.as_slice()));
                let status = output.status.code().unwrap_or(1);

                err!(status, message);
            }
        })
    }

    /// Runs the command and passes its output through the context streams.
    ///
    /// ```
    /// use crate::app;
    /// use crate::util::run;
    /// use structopt::StructOpts;
    ///
    /// let app = app::Application::from_args();
    /// let mut context = app::ApplicationContext::new(&app);
    ///
    /// run::Run::new("aws")
    ///     .with_aws_options(&context)
    ///     .arg("eks")
    ///     .arg("list-clusters")
    ///     .pass_through(&mut context)?;
    /// ```
    pub fn pass_through(&mut self, context: &Application) -> Result<()> {
        if !in_path(&self.program)? {
            err!(1, "The program, {}, could be found in PATH.", self.program);
        }

        Runtime::new()?.block_on(async {
            let mut child = self
                .builder
                .stderr(Stdio::piped())
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .spawn()?;

            let stderr = child.stderr.take();
            let stdout = child.stdout.take();

            let (result, _, _) = join!(
                child.wait(),
                async {
                    if let Some(mut source) = stderr {
                        let mut buffer = vec![0];
                        let mut target = context.error();

                        loop {
                            match source.read(&mut buffer).await {
                                Ok(0) => break,
                                Ok(_) => {
                                    target.write_all(&buffer)?;
                                    target.flush()?;
                                }
                                Err(error) => err!(1, "{}", error),
                            }
                        }
                    }

                    Ok(())
                },
                async {
                    if let Some(mut source) = stdout {
                        let mut buffer = vec![0];
                        let mut target = context.output();

                        loop {
                            match source.read(&mut buffer).await {
                                Ok(0) => break,
                                Ok(_) => {
                                    target.write_all(&buffer)?;
                                    target.flush()?;
                                }
                                Err(error) => err!(1, "{}", error),
                            }
                        }
                    }

                    Ok(())
                }
            );

            let status = result?;

            if !status.success() {
                err!(status.code().unwrap_or(1));
            }

            Ok(())
        })
    }

    /// Assumes that the AWS CLI is being invoked and adds additional arguments.
    ///
    /// The given context will be used to add the `--profile` and `--region` options for the AWS
    /// CLI if the user has provided any. This will allow the AWS CLI to inherit any profile and
    /// region information provided to this application.
    ///
    /// ```
    /// use crate::app;
    /// use structopt::StructOpts;
    ///
    /// let app = app::Application::from_args();
    /// let context = app::ApplicationContext::new(&app);
    ///
    /// Run::new("aws")
    ///     .with_aws_options(&context)
    ///     .arg("configure")
    ///     .arg("get")
    ///     .arg("sso_start_url")
    /// ```
    pub fn with_aws_options(&mut self, context: &Application) -> &mut Self {
        if let Some(profile) = context.profile() {
            self.arg("--profile").arg(profile);
        }

        if let Some(region) = context.region() {
            self.arg("--region").arg(region);
        }

        self
    }
}

/// Checks if a program can be found in `PATH`.
fn in_path(program: &str) -> Result<bool> {
    let mut cache = match CHECK_CACHE.lock() {
        Ok(cache) => cache,
        Err(error) => {
            return Err(Error::new(1)
                .message(format!("{}", error))
                .context("Could not acquire lock on CHECK_CACHE.".to_owned()))
        }
    };

    if let Some(status) = cache.get(program) {
        Ok(*status)
    } else {
        let status = which(program).is_ok();

        cache.insert(program.to_owned(), status);

        Ok(status)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(unix)]
    use std::io::Seek;

    #[test]
    fn argument_building() {
        let args = Run::new("test")
            .arg("arg1")
            .arg("arg2")
            .arg("arg3")
            .get_args()
            .to_owned();

        assert_eq!(args.as_ref(), vec!["arg1", "arg2", "arg3"]);
    }

    #[test]
    fn aws_options_added() {
        let context = Application::test(Some("profile".to_owned()), Some("region".to_owned()));

        let args = Run::new("aws")
            .with_aws_options(&context)
            .get_args()
            .to_owned();

        assert_eq!(
            args.as_ref(),
            vec!["--profile", "profile", "--region", "region"]
        );
    }

    #[cfg(unix)]
    #[test]
    fn collect_output() {
        let result = Run::new("printf").arg("Hello, %s!").arg("world").output();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");
    }

    #[test]
    fn collect_output_not_in_path() {
        let result = Run::new("does-not-exist").output();

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "The program, does-not-exist, could be found in PATH.\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn found_in_path() {
        assert!(in_path("printf").unwrap());

        let cache = CHECK_CACHE.lock().unwrap();

        assert!(cache.contains_key("printf"));
        assert!(cache.get("printf").unwrap());
    }

    #[test]
    fn not_found_in_path() {
        assert!(!in_path("does-not-exist").unwrap());

        let cache = CHECK_CACHE.lock().unwrap();

        assert!(cache.contains_key("does-not-exist"));
        assert!(!cache.get("does-not-exist").unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn pass_through_output() {
        let context = Application::test(None, None);

        let result = Run::new("printf")
            .arg("Hello, %s!")
            .arg("world")
            .pass_through(&context);

        let mut output = context.output();

        output.rewind().unwrap();

        assert!(result.is_ok());
        assert_eq!(output.to_string_lossy(), "Hello, world!");
    }

    #[test]
    fn pass_through_output_not_in_path() {
        let context = Application::test(None, None);
        let result = Run::new("does-not-exist").pass_through(&context);

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "The program, does-not-exist, could be found in PATH.\n"
        );
    }
}
