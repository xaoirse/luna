use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Time(pub DateTime<Utc>);
impl FromStr for Time {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Utc::now() - chrono::Duration::hours(s.parse()?)))
    }
}

impl Default for Time {
    fn default() -> Self {
        Self(Utc::now())
    }
}
