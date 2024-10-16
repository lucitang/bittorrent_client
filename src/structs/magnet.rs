use anyhow::Context;
use hex::FromHex;
use reqwest::Url;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MagnetLink {
    pub info_hash: [u8; 20],
    pub name: String,
    pub tracker_url: Url,
}

impl FromStr for MagnetLink {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        let xt = url
            .query_pairs()
            .find(|(key, _)| key == "xt")
            .and_then(|(_, value)| value.strip_prefix("urn:btih:").map(|v| v.to_string()))
            .context("Retrieving info hash value")?;
        let bytes = Vec::from_hex(xt)?;

        println!("Bytes length {}", bytes.len());
        let info_hash: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .context("Converting bytes to [u8;40] array")?;

        let name = url
            .query_pairs()
            .find(|(key, _)| key == "dn")
            .unwrap()
            .1
            .to_string();

        let url_value = url.query_pairs().find(|(key, _)| key == "tr").unwrap().1;
        let tracker_url: Url = Url::from_str(&url_value)?;

        Ok(MagnetLink {
            info_hash,
            name,
            tracker_url,
        })
    }
}
