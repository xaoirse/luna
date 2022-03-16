use addr::parse_domain_name;
use itertools::Itertools;
use std::fmt::{self, Display};

pub fn dnsgen(subdomain: String, wl: Vec<String>) -> Vec<String> {
    let domain = if let Ok(d) = parse_domain_name(&subdomain) {
        d
    } else {
        return vec![];
    };

    let domain = if let Some(d) = domain.root() {
        d
    } else {
        return vec![];
    };

    let sub = subdomain.replace(domain, "");

    if sub.is_empty() {
        return vec![];
    }

    let split = Sub::parse(sub);

    let mut subs = vec![];
    subs.extend(split.clone().change(wl.clone()));
    subs.extend(split.clone().variety());
    subs.extend(split.permutation(wl));

    let mut subs = subs
        .into_iter()
        .map(|mut p| {
            p.push('.');
            p.push_str(domain);
            p
        })
        .collect_vec();
    subs.sort();
    subs.dedup();
    if let Ok(i) = subs.binary_search(&subdomain) {
        subs.swap_remove(i);
    }
    subs
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Prefix {
    None,
    Alph,
    Num,
    Dash,
    Dot,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Segment {
    prefix: Prefix,
    str: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Sub {
    segments: Vec<Segment>,
}

impl Sub {
    fn parse(sub: String) -> Self {
        let mut segments = vec![];
        let mut chunc = vec![];

        let mut prefix = Prefix::None;
        let mut char = Prefix::None;

        for c in sub.chars() {
            if c.is_alphabetic() && char != Prefix::Alph {
                if !chunc.is_empty() {
                    segments.push(Segment {
                        prefix,
                        str: std::mem::take(&mut chunc).into_iter().collect(),
                    });
                }

                prefix = char;
                char = Prefix::Alph
            } else if c.is_numeric() && char != Prefix::Num {
                if !chunc.is_empty() {
                    segments.push(Segment {
                        prefix,
                        str: std::mem::take(&mut chunc).into_iter().collect(),
                    });
                }
                prefix = char;
                char = Prefix::Num
            } else if c == '-' {
                char = Prefix::Dash;
                continue;
            } else if c == '.' {
                char = Prefix::Dot;
                continue;
            }
            chunc.push(c);
        }
        if !chunc.is_empty() {
            segments.push(Segment {
                prefix,
                str: std::mem::take(&mut chunc).into_iter().collect(),
            });
        }

        Sub { segments }
    }

    fn change(self, wl: Vec<String>) -> Vec<String> {
        let mut subs = vec![];

        for i in 0..self.segments.len() {
            let first_char = self.segments[i].str.chars().next().unwrap();
            if first_char.is_numeric() {
                for j in 0..10 {
                    let mut tmp = self.segments.clone();
                    tmp[i].str.replace_range(
                        self.segments[i].str.len() - 1..self.segments[i].str.len(),
                        &j.to_string(),
                    );
                    subs.push(tmp);
                }
            } else if first_char.is_alphabetic() {
                for w in wl.clone() {
                    let mut tmp = self.segments.clone();
                    tmp[i].str = w;
                    subs.push(tmp);
                }
            }
        }

        subs.into_iter()
            .map(|s| Sub { segments: s }.to_string())
            .collect_vec()
    }

    fn permutation(mut self, wl: Vec<String>) -> Vec<String> {
        let mut n = self.segments.len();
        let mut results = vec![];

        results.extend(
            self.segments
                .iter()
                .map(|s| s.to_owned())
                .permutations(n)
                .unique()
                .map(|s| Sub { segments: s }.to_string())
                .collect_vec(),
        );
        n += 1;
        for w in wl {
            for pre in [Prefix::Dash, Prefix::Dot, Prefix::None] {
                self.segments.push(Segment {
                    prefix: pre,
                    str: w.clone(),
                });

                results.extend(
                    self.segments
                        .iter()
                        .map(|s| s.to_owned())
                        .permutations(n)
                        .unique()
                        .map(|s| Sub { segments: s }.to_string())
                        .collect_vec(),
                );
            }
        }

        results
    }

    pub fn variety(self) -> Vec<String> {
        let n = self.segments.len();
        let perms = self
            .segments
            .into_iter()
            .permutations(n)
            .unique()
            .collect_vec();

        let delims = ["", ".", "-"]
            .into_iter()
            .permutations(n - 1)
            .unique()
            .collect_vec();

        let mut subs = vec![];

        for perm in &perms {
            for delim in &delims {
                let mut sub = String::new();
                for i in 0..n {
                    sub.push_str(&perm[i].str);
                    if i < n - 1 {
                        sub.push_str(delim[i]);
                    }
                }
                subs.push(sub);
            }
        }
        subs
    }
}

impl Display for Sub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        for (i, s) in self.segments.iter().enumerate() {
            if i == 0 {
                str.push_str(&s.str);
            } else {
                match s.prefix {
                    Prefix::Alph => str.push_str(&s.str),
                    Prefix::Num => str.push_str(&s.str),
                    Prefix::None => str.push_str(&s.str),
                    Prefix::Dot => str.push_str(&format!(".{}", s.str)),
                    Prefix::Dash => str.push_str(&format!("-{}", s.str)),
                }
            }
        }

        write!(f, "{}", str)
    }
}

mod test {
    #[test]
    fn split() {
        use super::*;

        assert_eq!(
            Sub::parse("ab12cd.efg".to_string()),
            Sub {
                segments: vec![
                    Segment {
                        prefix: Prefix::None,
                        str: "ab".to_string()
                    },
                    Segment {
                        prefix: Prefix::Alph,
                        str: "12".to_string()
                    },
                    Segment {
                        prefix: Prefix::Num,
                        str: "cd".to_string()
                    },
                    Segment {
                        prefix: Prefix::Dot,
                        str: "efg".to_string()
                    }
                ]
            }
        );

        assert_eq!(
            Sub::parse("-abc-123.efg".to_string()),
            Sub {
                segments: vec![
                    Segment {
                        prefix: Prefix::Dash,
                        str: "abc".to_string()
                    },
                    Segment {
                        prefix: Prefix::Dash,
                        str: "123".to_string()
                    },
                    Segment {
                        prefix: Prefix::Dot,
                        str: "efg".to_string()
                    }
                ]
            }
        );
    }
}
