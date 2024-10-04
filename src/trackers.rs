use crate::structs::peers::PeerList;
use reqwest::{Client, Error, Url};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize)]
pub struct QueryParams {
    /// the info hash of the torrent
    /// 20 bytes long, will need to be URL encoded
    /// Note: this is NOT the hexadecimal representation, which is 40 bytes long
    // pub info_hash: String,

    ///  A unique identifier for your client
    ///  A string of length 20 that you get to pick. You can use something like 00112233445566778899
    pub peer_id: String,

    /// The port your client is listening on
    /// You can set this to 6881, you will not have to support this functionality during this challenge.
    pub port: u16,

    /// The total amount uploaded so far
    /// Since your client hasn't uploaded anything yet, you can set this to 0.
    pub uploaded: u64,

    /// The total amount downloaded so far
    /// Since your client hasn't downloaded anything yet, you can set this to 0.
    pub downloaded: u64,

    /// The number of bytes left to download
    /// Since you client hasn't downloaded anything yet, this'll be the total length of the file (you've extracted this value from the torrent file in previous stages)
    pub left: u64,

    /// Whether the peer list should use the compact representation
    /// For the purposes of this challenge, set this to 1.
    /// The compact representation is more commonly used in the wild, the non-compact representation is mostly supported for backward-compatibility.
    pub compact: u8,
}

#[derive(Deserialize, Debug)]
pub struct TrackerResponse {
    /// An integer, indicating how often your client should make a request to the tracker.
    /// You can ignore this value for the purposes of this challenge.
    pub interval: u64,

    /// A string, which contains list of peers that your client can connect to.
    /// Each peer is represented using 6 bytes. The first 4 bytes are the peer's IP address and the last 2 bytes are the peer's port number.
    pub peers: PeerList,
}

/// Get the tracker information
pub async fn get_tracker_info(
    endpoint: &str,
    query_params: QueryParams,
    info_hash: String,
) -> Result<TrackerResponse, Error> {
    // Create a reqwest client
    let client = Client::new();
    let mut url = Url::from_str(endpoint).expect("Parsing tracker URL");

    let encoded_req = serde_urlencoded::to_string(query_params).unwrap();
    url.set_query(Some(
        format!("{encoded_req}&info_hash={}", info_hash).as_str(),
    ));
    // println!("URL: {:?}", url);
    let response = client.get(url).send().await?;
    let response = response.bytes().await?;
    let response: TrackerResponse =
        serde_bencode::from_bytes::<TrackerResponse>(&response).expect("Parsing tracker response");

    // println!("Response: {:?}", response);

    Ok(response)
}
