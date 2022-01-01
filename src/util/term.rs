//! Provides simplified APIs for interacting with the terminal and generating interfaces.

use crate::{app, err};
use std::fmt;

/// Prompts the user to select an item from a list.
///
/// This function will draw an interface that will display a prompt, followed by a list of items
/// for the user to select. Once the user has made their selection, the selected option will be
/// returned.
///
/// ```
/// use crate::util::term::select;
///
/// let choices = vec!["a", "b", "c"];
/// let selected = select("Please choose a letter:", &choices);
///
/// println!("You chose: {}", selected);
/// ```
pub fn select<'a, 'b, T>(prompt: &'a str, list: &'b [T]) -> app::Result<&'b T>
where
    T: fmt::Display,
{
    use requestty::{prompt_one, ErrorKind, Question};

    let compatible = list
        .iter()
        .map(|i| format!("{}", i))
        .collect::<Vec<String>>();

    let question = Question::select("menu")
        .message(prompt)
        .choices(compatible)
        .default(0)
        .build();

    let index = match prompt_one(question) {
        Ok(answer) => answer.as_list_item().unwrap().index,
        Err(ErrorKind::Eof) => err!(1, "Unexpected input provided."),
        Err(ErrorKind::Interrupted) => err!(1, "Prompt was canceled."),
        Err(ErrorKind::IoError(error)) => err!(error.raw_os_error().unwrap_or(1), "{}", error),
    };

    Ok(&list[index])
}
