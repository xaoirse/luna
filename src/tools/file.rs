use crate::alert::Alert;
use std::io::SeekFrom;
use std::io::{Read, Seek, Write};

type Data = std::collections::HashMap<String, Vec<String>>;
pub trait Commands {
    fn commands(self, file_name: &str) -> Vec<String>;
}
impl Commands for Data {
    fn commands(self, file_name: &str) -> Vec<String> {
        let path = match crate::env::get("PATH") {
            Some(path) => format!("{}/{}", path, file_name),
            None => file_name.to_string(),
        };
        let file = std::fs::read_to_string(path).unwrap();

        // Extract command lines
        let vec = file
            .lines()
            .filter(|line| !line.is_empty() && !line.trim().starts_with("#"))
            .map(|line| line.to_string())
            .collect();

        // Replace with data
        replace(vec, self)
    }
}

fn replace(vec: Vec<String>, data: Data) -> Vec<String> {
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

pub fn _save(file_name: &str, text: &str) {
    let path = match crate::env::get("PATH") {
        Some(path) => path,
        None => "luna".to_string(),
    };
    match std::fs::OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(format!("{}/{}", path, file_name))
    {
        Ok(mut file) => {
            // Check for duplicates in file
            let mut f = String::new();
            file.read_to_string(&mut f).unwrap();
            for l in f.lines() {
                if l == text {
                    return;
                }
            }

            // Insert newline if needed
            let mut buf: [u8; 1] = [0];
            if let Ok(_) = file.seek(SeekFrom::End(-1)) {
                if let Ok(_) = file.read(&mut buf) {
                    if buf[0] != b'\n' {
                        file.write(b"\n").unwrap();
                    }
                }
            }

            file.write(text.as_bytes()).unwrap();
        }
        Err(err) => err.error(),
    }
}

pub fn _exists(file_name: &str, text: &str) -> bool {
    let path = match crate::env::get("PATH") {
        Some(path) => path,
        None => "luna".to_string(),
    };

    match std::fs::OpenOptions::new()
        .read(true)
        .open(format!("{}/{}", path, file_name))
    {
        Ok(mut file) => {
            let mut f = String::new();
            file.read_to_string(&mut f).unwrap();
            for l in f.lines() {
                if l == text {
                    format!("{}/{}", path, file_name).found();
                    return true;
                }
            }
            format!("{}/{}", path, file_name).not_found();
            false
        }
        Err(err) => {
            err.error();
            false
        }
    }
}
