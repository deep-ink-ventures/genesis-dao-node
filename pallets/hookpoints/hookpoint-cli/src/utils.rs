pub fn camel_case_to_kebap(s: &str) -> String {
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
