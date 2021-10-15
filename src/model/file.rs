use crate::alert;
use dotenv_codegen::*;
use std::io::SeekFrom;
use std::io::{Read, Seek, Write};

static PATH: &str = dotenv!("DOMAINS_PATH");

pub fn save(file_name: &str, text: &str) -> bool {
    match std::fs::OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(format!("{}/{}", PATH, file_name))
    {
        Ok(mut file) => {
            // Check for duplicates in file
            let mut f = String::new();
            file.read_to_string(&mut f).unwrap();
            for l in f.lines() {
                if l == text {
                    return false;
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
            true
        }
        Err(err) => {
            alert::nok(&err.to_string());
            false
        }
    }
}

pub fn exists(file_name: &str, text: &str) -> bool {
    match std::fs::OpenOptions::new()
        .read(true)
        .open(format!("{}/{}", PATH, file_name))
    {
        Ok(mut file) => {
            let mut f = String::new();
            file.read_to_string(&mut f).unwrap();
            for l in f.lines() {
                if l == text {
                    alert::found(format!("{}/{}", PATH, file_name));
                    return true;
                }
            }
            alert::nfound(format!("{}/{}", PATH, file_name));
            false
        }
        Err(err) => {
            alert::nok(&err.to_string());
            false
        }
    }
}
