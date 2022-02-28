// https://serde.rs/custom-date-format.html

use chrono::{DateTime, Local, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

// The signature of a serialize_with function must follow the pattern:
//
//    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer
//
// although it may also be generic over the input types T.
pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(date) = date {
        let s = date.with_timezone(&Local::now().timezone()).to_rfc2822();
        serializer.serialize_str(&s)
    } else {
        let s = String::new();
        serializer.serialize_str(&s)
    }
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    match DateTime::parse_from_rfc2822(&s) {
        Ok(date) => Ok(Some(date.with_timezone(&Utc::now().timezone()))),
        Err(_) => Err(serde::de::Error::custom("Parse Error")),
    }
}
