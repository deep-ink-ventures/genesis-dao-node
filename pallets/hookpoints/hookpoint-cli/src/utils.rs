pub const INK_PRIMITIVES: &[&str; 2] = &["AccountId", "Hash"];
pub const INK_TYPES: [&str; 24] = [
    "bool",
    "u8", "u16", "u32", "u64", "u128",
    "i8", "i16", "i32", "i64", "i128",
    "Vec<u8>", "Vec<u16>", "Vec<u32>", "Vec<u64>", "Vec<u128>",
    "Vec<i8>", "Vec<i16>", "Vec<i32>", "Vec<i64>", "Vec<i128>",
    "AccountId",
    "Hash",
    "Balance",
];

pub fn get_default_for_ink_type(type_str: &str) -> String {
    match type_str {
        "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" | "Balance" => "0".to_string(),
        "Vec<u8>" | "Vec<u32>" | "Vec<u64>" | "Vec<u128>" | "Vec<i8>" | "Vec<i16>" | "Vec<i32>" | "Vec<i64>" | "Vec<i128>" => "vec![]".to_string(),
        "AccountId" => "AccountId::from([0x01; 32])".to_string(),
        "Hash" => "Hash::default()".to_string(),
        _ => panic!("Unknown INK type: {}", type_str),
    }
}

pub fn camel_case_to_kebab(s: &str) -> String {
    let mut kebap = String::new();
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        if chars[i].is_uppercase() {
            // Add a '-' if it's not the first character and previous character is not uppercase
            if i != 0 && !chars[i - 1].is_uppercase() {
                kebap.push('-');
            }
            kebap.push(chars[i].to_ascii_lowercase());
        } else {
            kebap.push(chars[i]);
        }
    }

    kebap
}

pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}
