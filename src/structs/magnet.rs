use anyhow::Context;
use hex::FromHex;
use reqwest::Url;
use std::borrow::Cow;
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
        let xt = url
            .query_pairs()
            .find_map(|(key, value)| {
                if key == "xt" {
                    value.strip_prefix("urn:btih:").map(|x| x.to_string())
                } else {
                    None
                }
            })
            .context("Retrieving info hash value")?;
        let bytes = Vec::from_hex(xt)?;

        println!("Bytes length {}", bytes.len());
        let info_hash: [u8; 20] = bytes
            .as_slice()
            .try_into()
            .context("Converting bytes to [u8;40] array")?;

        let name = url.query_pairs().find_map(|(key, value)| {
            if key == "dn" {
                Some(value.into_owned())
            } else {
                None
            }
        });

        let tracker_url = url.query_pairs().find_map(|(key, value)| {
            if key == "tr" {
                Some(
                    Url::from_str(&value.into_owned())
                        .context("Parsing tracker URL")
                        .unwrap(),
                )
            } else {
                None
            }
        });

        Ok(MagnetLink {
            info_hash,
            name,
            tracker_url,
        })
    }
}
