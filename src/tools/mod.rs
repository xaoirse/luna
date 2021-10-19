pub mod assetsfinder;
pub mod file;

pub type Data = std::collections::HashMap<String, Vec<String>>;

pub fn parse_file(path: &str, data: Data) -> Vec<String> {
    let file = std::fs::read_to_string(path).unwrap();

    // Extract command lines
    let mut vec = vec![];
    for line in file.lines() {
        if !line.is_empty() && !line.trim().starts_with("#") {
            vec.push(line.to_string());
        }
    }

    // Replace with data
    let commands = replace(vec, data);

    commands
}

pub fn replace(vec: Vec<String>, data: Data) -> Vec<String> {
    let mut tmp: Vec<String> = vec![];
    for v in vec {
        replace_recercive(v.to_string(), &mut tmp, &data);
    }
    tmp
}

fn replace_recercive(str: String, vec: &mut Vec<String>, data: &Data) {
    for (n, v) in data {
        if str.contains(n) {
            for i in v {
                let tmp = str.replace(n, i);
                replace_recercive(tmp.clone(), vec, &data);
            }
            return;
        }
    }
    vec.push(str);
}
