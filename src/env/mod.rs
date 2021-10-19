pub fn get(var: &str) -> Option<String> {
    match std::fs::read_to_string("luna.ini") {
        Ok(text) => {
            for line in text.lines() {
                if line.trim().starts_with(var) {
                    let (_, param) = line.split_once("=").unwrap_or(("", ""));
                    return Some(param.trim().to_string());
                }
            }
            None
        }
        Err(_) => None,
    }
}
