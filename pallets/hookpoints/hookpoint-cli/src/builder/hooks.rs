use std::collections::HashMap;
use crate::builder::mapper::ink_to_substrate;
use crate::config::models::Definitions;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn generate_function_signature(name: &str, arguments: &[(String, String)], return_type: &str) -> String {
    let args = arguments.iter()
        .map(|(arg_name, arg_type)| format!("{}: {}", arg_name, ink_to_substrate(arg_type)))
        .collect::<Vec<String>>()
        .join(", ");

    format!("pub fn {}<T: Config>(owner: T::AccountId, signer: T::AccountId, {}) -> {}", name, args, ink_to_substrate(return_type))
}

fn generate_function_body(name: &str, hook_point: &str, arguments: &[(String, String)], return_type: &str, return_default: &str) -> String {
   let hp_def = format!(r#"
   let hp = HP::<T>::create(
		"{}::{}",
		owner,
		signer
	)"#, name, hook_point);

    let mut args = arguments.iter()
        .map(|(arg_name, arg_type)| format!("\n\t\t.add_arg::<{}>({})", ink_to_substrate(arg_type), arg_name))
        .collect::<Vec<String>>()
        .join("");
    args.push_str(";");

    let execute = format!("\n\n\tHP::<T>::execute::<{}>(hp).unwrap_or({})", ink_to_substrate(return_type), return_default);
    format!("{} {}{}", hp_def, args, execute)
}

fn generate_hooks_rs(funcs: Vec<String>) -> String {
    let mut content: String = String::from("use crate::Config;\n");
    content.push_str("use pallet_hookpoints::Pallet as HP;\n\n");
    content.push_str(&funcs.join("\n\n"));
    content

}

pub fn create_hooks(config: Definitions) -> HashMap<String, String> {
    let mut pallet_to_hooks: HashMap<String, String> = HashMap::new();

    for (pallet_name, pallet_functions) in config.pallets {
        let mut funcs: Vec<String> = vec![];
        for pallet_function in pallet_functions {
            let args = pallet_function.arguments.iter().map(|arg| (arg.name.clone(), arg.type_.clone())).collect::<Vec<(String, String)>>();
            let function_signature = generate_function_signature(
                    &pallet_function.hook_point,
                    &args,
                &pallet_function.returns.type_
                );

            let function_body = generate_function_body(
                &config.name,
                &pallet_function.hook_point,
                &args,
               &pallet_function.returns.type_,
                &pallet_function.returns.default
            );

            funcs.push(format!("{}\n{{ {}\n}}", function_signature, function_body));
        }

        let mut content = String::from("use crate::Config;\nuse pallet_hookpoints::Pallet as HP;\n\n");
        content.push_str(&funcs.join("\n\n"));
        pallet_to_hooks.insert(pallet_name, content);
    }
    pallet_to_hooks
}
