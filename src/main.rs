use anyhow::{Context, Error};
use bittorrent_starter_rust::cli::{Cli, Commands};
use bittorrent_starter_rust::decoder::decode_bencoded_value;
use bittorrent_starter_rust::encoder::url_encode;
use bittorrent_starter_rust::torrent::Torrent;
use bittorrent_starter_rust::trackers;
use clap::Parser;
use rand::random;
use serde_bencode::from_bytes;
use std::fs;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();

    match args.subcmd {
        Commands::Decode { encoded_value } => {
            let (decoded_value, _) = decode_bencoded_value(&encoded_value);
            println!("{}", decoded_value.to_string());
        }
        Commands::Info { torrent_file } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);
            let torrent_hash = torrent.info_hash();
            println!("Info Hash: {}", hex::encode(torrent_hash));
            println!("Piece Length: {}", torrent.info.piece_length);
            println!("Piece Hashes:");
            for chunk in torrent.info.pieces.chunks(20) {
                println!("{:}", hex::encode(chunk));
            }
        }
        Commands::Peers { torrent_file } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let info = torrent.info_hash();
            let encoded_info = url_encode(&info);

            let query_params = trackers::QueryParams {
                peer_id: generate_peer_id(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: torrent.info.length as u64,
                compact: 1,
            };

            let tracker_response =
                trackers::get_tracker_info(&torrent.announce, query_params, encoded_info)
                    .await
                    .context("Getting tracker info")?;
            // println!("Tracker Response: {:?}", tracker_response);
            for peer in tracker_response.peers.0 {
                println!("{}", peer);
            }
        }
    };

    Ok(())
}

/// Generate a peer id on 20 characters
/// Ex: 47001398037243657525
fn generate_peer_id() -> String {
    let mut peer_id = String::new();
    for _ in 0..20 {
        let c = (random::<u8>() % 10) + 48; // 48 is the ASCII code for '0'
        peer_id.push(c as char);
    }
    peer_id
}
