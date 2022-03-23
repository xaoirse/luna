use super::*;

use serde::{self, Deserialize, Deserializer, Serializer};

pub fn serialize<S>(cidr: &IpCidr, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&cidr.to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<IpCidr, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.parse::<IpCidr>() {
        Ok(addr) => Ok(addr),
        Err(err) => Err(serde::de::Error::custom(err)),
    }
}
