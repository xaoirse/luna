use super::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    pub url: url::Url,
    pub title: Option<String>,
    pub sc: Option<String>,
    pub resp: Option<String>,
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        other.url[..url::Position::BeforePath] == self.url[..url::Position::BeforePath]
            && (self.url.path() == other.url.path()
                || match (other.url.path_segments(), self.url.path_segments()) {
                    (Some(a), Some(b)) => {
                        let a: HashSet<&str> = a
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();
                        let b: HashSet<&str> = b
                            .filter(|s| !s.is_empty() && s.parse::<usize>().is_err())
                            .collect();

                        let c = &a - &b;
                        let d = &b - &a;

                        c.len() < 2 && d.len() < 2
                    }
                    (None, None) => true,
                    _ => false,
                })
            && {
                if self.url.path().ends_with(".css") && other.url.path().ends_with(".css") {
                    return true;
                }
                if self.url.path().ends_with(".js") && other.url.path().ends_with(".js") {
                    return true;
                }
                true
            }
            && {
                let a: HashSet<_> = other
                    .url
                    .query_pairs()
                    .map(|(k, _)| k)
                    .filter(|s| !s.is_empty())
                    .collect();
                let b: HashSet<_> = self
                    .url
                    .query_pairs()
                    .map(|(k, _)| k)
                    .filter(|s| !s.is_empty())
                    .collect();

                a == b
            }
    }
}

impl Eq for Request {}

mod test {

    #[test]
    fn test_eq() {
        use super::super::AssetName;
        use std::str::FromStr;

        assert_eq!(
            AssetName::from_str("https://app.lexoffice.de/voucher/settings/template/popover/voucher/settings/text/text/lib/css/libs.css?v=c644c27a4").unwrap(),
            AssetName::from_str("https://app.lexoffice.de/voucher/settings/template/popover/voucher/settings/voucher/text/settings/voucher/settings/lib/css/libs.css?v=c644c27a4").unwrap()
        );

        assert_eq!(
            AssetName::from_str("https://memoryleaks.ir/tag/%d8%a2%d8%b3%db%8c%d8%a8-%d9%be%d8%b0%db%8c%d8%b1%db%8c-%d8%af%d8%b1-%d8%b1%d8%a8%d8%a7%d8%aa-%d9%87%d8%a7%db%8c-%d8%aa%d9%84%da%af%d8%b1%d8%a7%d9%85/feed/").unwrap(),
            AssetName::from_str("https://memoryleaks.ir/tag/%d8%af%d9%88%d8%b1-%d8%b2%d8%af%d9%86-%d9%85%da%a9%d8%a7%d9%86%db%8c%d8%b2%d9%85-%d8%aa%d8%b4%d8%ae%db%8c%d8%b5-%d8%af%d8%b3%d8%aa%da%af%d8%a7%d9%87-%d8%b1%d9%88%d8%aa-%d8%b4%d8%af%d9%87/feed/").unwrap()
        );

        assert_eq!(
            AssetName::from_str("http://aktuelles.haufe.de/eagle_kp_webapp/login/login.action;jsessionid=2acc8603184240a22e4fa5fe3525").unwrap(),
            AssetName::from_str("http://aktuelles.haufe.de/eagle_kp_webapp/login/login.action;jsessionid=2b0107dc883c28019e877799d07e").unwrap()
        );

        assert_eq!(
            AssetName::from_str("https://memoryleaks.ir/category/%d8%a7%d8%b1%d8%aa%d9%82%d8%a7%d8%b9-%d8%af%d8%b3%d8%aa%d8%b1%d8%b3%db%8c/feed/").unwrap(),
            AssetName::from_str("https://memoryleaks.ir/category/%d8%a7%d9%85%d9%86%db%8c%d8%aa-%d8%a7%d9%86%d8%af%d8%b1%d9%88%db%8c%d8%af/feed/").unwrap()
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
            AssetName::from_str("https://a.com/a/b/c/").unwrap(),
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
