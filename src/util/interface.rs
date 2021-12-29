//! Provides utilities for the command line interface.

use dialoguer::{theme::ColorfulTheme, Select};

/// Prompts the user to select a choice from a list.
pub fn choose<'a, 'b, S: ToString>(prompt: &'a str, items: &'b [S]) -> &'b S {
    let selected = Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .with_prompt(prompt)
        .default(0)
        .interact()
        .unwrap();

    &items[selected]
}
