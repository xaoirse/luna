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
    match s.split_once('/') {
        Some((ip_str, len_str)) => {
            let addr = match ip_str.parse::<IpAddr>() {
                Ok(addr) => addr,
                Err(err) => return Err(serde::de::Error::custom(err)),
            };
            let len = match len_str.parse::<u8>() {
                Ok(len) => len,
                Err(err) => return Err(serde::de::Error::custom(err)),
            };

            let cidr = match IpCidr::new(addr, len) {
                Ok(cidr) => cidr,
                Err(err) => return Err(serde::de::Error::custom(err)),
            };

            Ok(cidr)
        }
        None => Err(serde::de::Error::custom("Not a Cidr")),
    }
}
