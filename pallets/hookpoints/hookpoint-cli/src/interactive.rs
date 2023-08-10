extern crate dialoguer;
extern crate regex;

use dialoguer::{theme::ColorfulTheme, Select, Input};
use crate::config::models::{FunctionArgument, PalletFunction, ReturnValue};

pub const SNAKE_CASE_REGEX: &str = r"^[a-z]+(?:_[a-z]+)*$";
pub const CAMEL_CASE_REGEX: &str = r"^[A-Z][a-z]+(?:[A-Z][a-z]+)*$";

pub fn set_name() -> String {
    println!("\nEnter a name for your project (CamelCase please):\n");
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name (CamelCase please):")
        .validate_with({
            let camel_case_regex = regex::Regex::new(CAMEL_CASE_REGEX).unwrap();
            move |input: &String| -> Result<(), &str> {
                if camel_case_regex.is_match(input) {
                    Ok(())
                } else {
                    Err("This is not a valid name; please use CamelCase")
                }
            }
        })
        .interact_text()
        .unwrap()
}

pub fn select_pallet(pallets: Vec<String>) -> String {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a pallet to hook into:")
        .default(0)
        .items(&pallets)
        .interact()
        .unwrap();

    pallets[selection].to_string()
}

// Hook generation
// Extract the logic for getting the hook point name
pub fn get_hook_point_name() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a hook point name (snake_case please):")
        .validate_with({
            let snake_case_regex = regex::Regex::new(SNAKE_CASE_REGEX).unwrap();
            move |input: &String| -> Result<(), &str> {
                if snake_case_regex.is_match(input) {
                    Ok(())
                } else {
                    Err("This is not a valid hook point name; please use snake_case")
                }
            }
        }).interact_text().unwrap()
}

// Extract the logic for getting the argument details
pub fn get_argument_details(ink_primitives: &Vec<&str>) -> FunctionArgument {
    let arg_name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter an argument name (snake_case please):")
        .validate_with({
            let snake_case_regex = regex::Regex::new(SNAKE_CASE_REGEX).unwrap();
            move |input: &String| -> Result<(), &str> {
                if snake_case_regex.is_match(input) {
                    Ok(())
                } else {
                    Err("This is not a valid argument name; please use snake_case")
                }
            }
        }).interact_text().unwrap();

    let arg_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a type:")
        .default(0)
        .items(ink_primitives)
        .interact()
        .unwrap();

    FunctionArgument {
        name: arg_name,
        type_: ink_primitives[arg_type].to_string(),
    }
}

// Extract the logic for getting the return details
pub fn get_return_details(ink_primitives: &Vec<&str>, arguments: &Vec<FunctionArgument>, no_default_values: &Vec<&str>) -> ReturnValue {
    let return_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a return type:")
        .default(0)
        .items(ink_primitives)
        .interact()
        .unwrap();

    let mut arguments_with_return_type: Vec<&str> = arguments.iter()
        .filter(|arg| arg.type_ == ink_primitives[return_type])
        .filter(|arg| !no_default_values.contains(&arg.type_.as_str()))
        .map(|arg| arg.name.as_str())
        .collect();
    arguments_with_return_type.push(ink_primitives[return_type]);

    let return_default = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a default return value:")
        .default(0)
        .items(&arguments_with_return_type)
        .interact()
        .unwrap();

    ReturnValue {
        type_: ink_primitives[return_type].to_string(),
        default: arguments_with_return_type[return_default].to_string(),
    }
}

pub fn add_hook() -> PalletFunction {
    let hook_point = get_hook_point_name();

    let mut arguments: Vec<FunctionArgument> = Vec::new();
    let mut returns: Option<ReturnValue> = None;
    let ink_primitives = vec![
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "bool", "char", "str",
        "AccountId", "Balance",
    ];
    let no_default_values = vec!["AccountId"];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option:")
            .default(0)
            .items(&["Add a function argument", "Set a return value", "Save & create another hook ..."])
            .interact()
            .unwrap();

        match selection {
            0 => {
                let arg = get_argument_details(&ink_primitives);
                arguments.push(arg);
            }
            1 => {
                let ret = get_return_details(&ink_primitives, &arguments, &no_default_values);
                returns = Some(ret);
            }
            _ => break
        }
    }

    PalletFunction {
        hook_point,
        arguments,
        returns,
    }
}
