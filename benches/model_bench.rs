use std::str::FromStr;

use model::*;
pub fn insert(i: i32) {
    let mut luna = Luna::default();

    for i in 0..i {
        let pr = Program::from_str("Test").unwrap();
        let asset = Asset::from_str(&format!("test{i}.com")).unwrap();
        luna.insert_asset(asset, Some(pr)).unwrap();
    }
}
