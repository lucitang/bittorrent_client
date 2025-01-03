use anyhow::{anyhow, Context};
use hex::FromHex;
use reqwest::Url;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MagnetLink {
    pub info_hash: [u8; 20],
    pub name: Option<String>,
    pub tracker_url: String,
}

const XT_PREFIX: &'static str = "urn:btih:";

impl FromStr for MagnetLink {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        if url.scheme() != "magnet" {
            return Err(anyhow::anyhow!("Invalid scheme !"));
        }

        let query_pairs = url.query_pairs().collect::<HashMap<_, _>>();
        let xt = query_pairs.get("xt");

        let xt = xt
            .unwrap()
            .strip_prefix(XT_PREFIX)
            .ok_or(anyhow!("Invalid xt value"))?;
        let bytes = Vec::from_hex(xt)?;

        let info_hash: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .context("Info hash must be 20 bytes")?;

        let name = query_pairs.get("dn").map(|s| s.to_string());

        let tracker_url = query_pairs
            .get("tr")
            .map(|s| Url::from_str(&s))
            .transpose()?;

        let tracker_url = tracker_url.map_or(String::new(), |s| s.to_string());

        Ok(MagnetLink {
            info_hash,
            name,
            tracker_url,
        })
    }
}
