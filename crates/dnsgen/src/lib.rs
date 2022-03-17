use addr::parse_domain_name;
use std::fmt::{self, Display};

pub fn dnsgen(subdomains: Vec<String>, wl: Vec<String>) -> Vec<String> {
    let mut results = vec![];

    for subdomain in subdomains.clone() {
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

        let subs = Sub::parse(sub).build(wl.clone());

        let subs = subs
            .into_iter()
            .map(|mut p| {
                p.push_str(domain);
                p
            })
            .collect::<Vec<_>>();

        results.extend(subs);
    }

    results.sort();
    results.dedup();

    for subdomain in subdomains {
        if let Ok(i) = results.binary_search(&subdomain) {
            results.swap_remove(i);
        }
    }

    results
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum Segment {
    Alph(String),
    Num(String),
    Char(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Sub {
    segments: Vec<Segment>,
}

impl Sub {
    fn parse(sub: String) -> Self {
        let mut segments = vec![];
        let mut segment = Segment::Alph(sub.clone());

        for (i, c) in sub.chars().enumerate() {
            if i == 0 {
                if c.is_alphabetic() {
                    segment = Segment::Alph(c.to_string());
                } else if c.is_numeric() {
                    segment = Segment::Num(c.to_string());
                } else {
                    segment = Segment::Char(c.to_string());
                }
                continue;
            }
            if c.is_alphabetic() {
                if let Segment::Alph(str) = &mut segment {
                    str.push(c);
                } else {
                    segments.push(segment);
                    segment = Segment::Alph(c.to_string());
                }
            } else if c.is_numeric() {
                if let Segment::Num(str) = &mut segment {
                    str.push(c);
                } else {
                    segments.push(segment);
                    segment = Segment::Num(c.to_string());
                }
            } else if let Segment::Char(str) = &mut segment {
                str.push(c);
            } else {
                segments.push(segment);
                segment = Segment::Char(c.to_string());
            }
        }

        segments.push(segment);

        Sub { segments }
    }

    fn build(self, wl: Vec<String>) -> Vec<String> {
        let mut subs = vec![];

        for i in 0..self.segments.len() {
            for w in wl.clone() {
                match self.segments[i].clone() {
                    Segment::Num(s) => {
                        for j in 0..10u8 {
                            let mut copy = self.segments.clone();
                            let mut s = s.clone();
                            s.replace_range(s.len() - 1..s.len(), &j.to_string());
                            copy[i] = Segment::Alph(s);
                            subs.push(copy.clone());
                        }
                        for pre in ["", "-", "."] {
                            let mut tmp = self.segments.clone();
                            tmp[i] = Segment::Alph(format!("{}{}{}", w, pre, &s));
                            subs.push(tmp);

                            let mut tmp = self.segments.clone();
                            tmp[i] = Segment::Alph(format!("{}{}{}", &s, pre, w));
                            subs.push(tmp);

                            let mut tmp = self.segments.clone();
                            tmp[i] = Segment::Alph(w.to_string());
                            subs.push(tmp);
                        }
                    }
                    Segment::Alph(s) => {
                        for pre in ["", "-", "."] {
                            let mut tmp = self.segments.clone();
                            tmp[i] = Segment::Alph(format!("{}{}{}", w, pre, &s));
                            subs.push(tmp);

                            let mut tmp = self.segments.clone();
                            tmp[i] = Segment::Alph(format!("{}{}{}", &s, pre, w));
                            subs.push(tmp);

                            let mut tmp = self.segments.clone();
                            tmp[i] = Segment::Alph(w.to_string());
                            subs.push(tmp);
                        }
                    }
                    Segment::Char(_) => (),
                }
            }
        }

        subs.into_iter()
            .map(|s| Sub { segments: s }.to_string())
            .collect()
    }
}

impl Display for Sub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        self.segments.iter().for_each(|s| match s {
            Segment::Alph(s) => str.push_str(s),
            Segment::Num(s) => str.push_str(s),
            Segment::Char(s) => str.push_str(s),
        });

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
                    Segment::Alph("ab".to_string()),
                    Segment::Num("12".to_string()),
                    Segment::Alph("cd".to_string()),
                    Segment::Char(".".to_string()),
                    Segment::Alph("efg".to_string()),
                ]
            }
        );
    }
}
