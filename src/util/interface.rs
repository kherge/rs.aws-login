//! Provides utilities for the command line interface.

use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

/// Prompts the user to select a choice from a list.
pub fn choose<S: ToString>(items: &[S]) -> &S {
    let mut selected: Option<usize> = None;

    while selected.is_none() {
        selected = Select::with_theme(&ColorfulTheme::default())
            .items(items)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
    }

    &items[selected.unwrap()]
}
