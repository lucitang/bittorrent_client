use anyhow::{anyhow, Context, Error};
use hex::FromHex;
use reqwest::Url;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MagnetLink {
    pub info_hash: [u8; 20],
    pub name: Option<String>,
    pub tracker_url: Option<Url>,
}

impl FromStr for MagnetLink {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        if url.scheme() != "magnet" {
            return Err(anyhow::anyhow!("Invalid scheme !"));
        }

        let query_pairs = url.query_pairs().collect::<HashMap<_, _>>();
        let xt = query_pairs.get("xt");

        if xt.is_none() {
            return Err(anyhow::anyhow!("Info hash required"));
        }
        let xt = xt
            .unwrap()
            .strip_prefix("urn:btih:")
            .ok_or(anyhow!("Missing xt prefix"))?;
        let bytes = Vec::from_hex(xt)?;

        let info_hash: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .context("Converting bytes to [u8;40] array")?;

        let name = query_pairs.get("dn").map(|s| s.to_string());

        let tracker_url = query_pairs
            .get("tr")
            .map(|s| Url::from_str(&s))
            .transpose()?;

        Ok(MagnetLink {
            info_hash,
            name,
            tracker_url,
        })
    }
}
