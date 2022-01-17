//! Provides macros that are commonly used through the application.

/// A macro shortcut for writing to the context error stream.
///
/// This macro simplifies the process of acquiring a lock on the error output stream from
/// [`Context`] so that it can be written to. The macro takes the same form as [`writeln`],
/// except that it accepts the `Context` object as the first argument.
#[macro_export]
macro_rules! errorln {
    ($context:expr, $message:expr) => {{
        crossterm::queue!(
            $context.error(),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
            crossterm::style::Print($message),
            crossterm::style::ResetColor,
            crossterm::cursor::MoveToNextLine(1),
        )
    }};
    ($context:expr, $message:tt, $($args:tt)*) => {{
        crossterm::queue!(
            $context.error(),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
            crossterm::style::Print(format!($message, $($args)*)),
            crossterm::style::ResetColor,
            crossterm::cursor::MoveToNextLine(1),
        )
    }};
}

/// A macro shortcut for writing to the context output stream.
///
/// This macro simplifies the process of acquiring a lock on the standard output stream from
/// [`Context`] so that it can be written to. The macro takes the same form as [`writeln`],
/// except that it accepts the `Context` object as the first argument.
#[macro_export]
macro_rules! outputln {
    ($context:expr, $message:expr) => {{
        use std::io::Write;

        writeln!($context.output(), $message)
    }};
    ($context:expr, $message:tt, $($args:tt)*) => {{
        use std::io::Write;

        writeln!($context.output(), $message, $($args)*)
    }};
}
