use model::*;
use std::{
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

pub fn dedup(n: i32) {
    let mut luna = Luna::default();

    for i in 0..n {
        let l = Luna {
            programs: vec![Program {
                name: "".to_string(),
                scopes: vec![Scope {
                    asset: ScopeType::from_str("test.com").unwrap(),
                    subs: vec![Sub {
                        urls: vec![Url::from_str(&format!("https://luna.test?{}", i)).unwrap()],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        luna.append(l);
    }

    let term = Arc::new(AtomicBool::new(false));
    luna.dedup(term);
}
