use super::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    pub url: url::Url,
    pub title: Option<String>,
    pub sc: Option<String>,
    pub resp: Option<String>,
}

impl FromStr for Request {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            url: url::Url::from_str(s)?,
            title: None,
            sc: None,
            resp: None,
        })
    }
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        other.url[..url::Position::BeforePath] == self.url[..url::Position::BeforePath]
            && match (self.url.path_segments(), other.url.path_segments()) {
                (None, None) => true,
                (None, Some(_)) => false,
                (Some(_), None) => false,
                (Some(s_path), Some(o_path)) => {
                    let s_path: Vec<_> = s_path
                        .filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                        .collect();
                    let o_path: Vec<_> = o_path
                        .filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                        .collect();

                    if s_path.len() != o_path.len() {
                        false
                    } else {
                        let a: HashSet<_> = other
                            .url
                            .query_pairs()
                            .map(|(k, _)| k)
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();
                        let b: HashSet<_> = self
                            .url
                            .query_pairs()
                            .map(|(k, _)| k)
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();

                        let diff = if !(a.is_empty() ^ b.is_empty())
                            && (a.is_subset(&b) || b.is_subset(&a))
                        {
                            2
                        } else {
                            1
                        };

                        s_path
                            .iter()
                            .zip(o_path.iter())
                            .filter(|(s, o)| s != o)
                            .count()
                            < diff
                    }
                }
            }
    }
}

impl Eq for Request {}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.url[..url::Position::BeforePath] != other.url[..url::Position::BeforePath] {
            return self.url[..url::Position::BeforePath]
                .cmp(&other.url[..url::Position::BeforePath]);
        }

        match (self.url.path_segments(), other.url.path_segments()) {
            (None, None) => (),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(s_path), Some(o_path)) => {
                let s_path: Vec<_> = s_path
                    .filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                    .collect();
                let o_path: Vec<_> = o_path
                    .filter(|p| !p.is_empty() && p.parse::<usize>().is_err())
                    .collect();

                match s_path.len().cmp(&o_path.len()) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => {
                        let a: HashSet<_> = other
                            .url
                            .query_pairs()
                            .map(|(k, _)| k)
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();
                        let b: HashSet<_> = self
                            .url
                            .query_pairs()
                            .map(|(k, _)| k)
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();

                        let diff = if !(a.is_empty() ^ b.is_empty())
                            && (a.is_subset(&b) || b.is_subset(&a))
                        {
                            1
                        } else {
                            0
                        };

                        let diffs: Vec<_> = s_path
                            .iter()
                            .zip(o_path.iter())
                            .filter(|(s, o)| s != o)
                            .collect();
                        if diffs.len() > diff {
                            return diffs[0].0.cmp(diffs[0].1);
                        }
                    }
                }
            }
        }

        Ordering::Equal
    }
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

mod test {

    #[test]
    fn asset_cmp() {
        use super::*;

        assert!(
            Request::from_str("http://a.com/x/y/").unwrap()
                == Request::from_str("http://a.com/x/y//").unwrap()
        );

        assert!(
            Request::from_str("http://a.com/").unwrap()
                < Request::from_str("http://a.com/x/y/z/").unwrap()
        );

        assert!(!Request::from_str("http://a.com/x/y/")
            .unwrap()
            .eq(&Request::from_str("http://a.com/x/y/z/").unwrap()));
    }

    #[test]
    fn test_eq() {
        use super::super::AssetName;
        use std::str::FromStr;

        assert_ne!(
            AssetName::from_str("https://test.com/voucher/settings/template/popover/voucher/settings/text/text/lib/css/libs.css?v=c644c27a4").unwrap(),
            AssetName::from_str("https://test.com/voucher/settings/template/popover/voucher/settings/voucher/text/settings/voucher/settings/lib/css/libs.css?v=c644c27a4").unwrap()
        );

        assert_eq!(
            AssetName::from_str("https://test.com/tag/a/feed/").unwrap(),
            AssetName::from_str("https://test.com/tag/b/feed/").unwrap()
        );

        assert_eq!(
            AssetName::from_str("http://test.com/eagle_kp_webapp/login/login.action;jsessionid=2acc8603184240a22e4fa5fe3525").unwrap(),
            AssetName::from_str("http://test.com/eagle_kp_webapp/login/login.action;jsessionid=2b0107dc883c28019e877799d07e").unwrap()
        );

        assert_eq!(
            AssetName::from_str("https://test.com/category/a/feed/").unwrap(),
            AssetName::from_str("https://test.com/category/b/feed/").unwrap()
        );

        assert_ne!(
            AssetName::from_str("https://a.com").unwrap(),
            AssetName::from_str("https://b.com").unwrap()
        );
        assert_eq!(
            AssetName::from_str("https://a.com/").unwrap(),
            AssetName::from_str("https://a.com").unwrap()
        );

        assert_eq!(
            AssetName::from_str("https://a.com/a/b/c/4").unwrap(),
            AssetName::from_str("https://a.com/a/d/c").unwrap()
        );
        assert_ne!(
            AssetName::from_str("https://a.com/a/b/c/").unwrap(),
            AssetName::from_str("https://a.com/a/d/e").unwrap()
        );

        assert_ne!(
            AssetName::from_str("https://a.com/a/23/b").unwrap(),
            AssetName::from_str("https://b.com/b/a/23/a/").unwrap()
        );
    }
}
