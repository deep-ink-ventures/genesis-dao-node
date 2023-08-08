extern crate dialoguer;

use std::path::Path;
use dialoguer::{theme::ColorfulTheme, Select};

pub fn select_pallet(pallets: Vec<String>) -> String {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a pallet to hook into:")
        .default(0)
        .items(&pallets)
        .interact()
        .unwrap();

    pallets[selection].to_string()
}