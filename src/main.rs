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
use std::io::{Read, Write};
use std::net::TcpStream;

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
        Commands::Handshake { torrent_file, peer } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let protocol_byte: &u8 = &19;
            let protocol: &[u8; 19] = b"BitTorrent protocol";
            let reserved_bytes: [u8; 8] = [0; 8];
            let info_hash = torrent.info_hash();
            let peer_id = "00112233445566778899";

            let mut handshake_base: Vec<u8> = Vec::new();
            handshake_base.push(*protocol_byte);
            handshake_base.extend_from_slice(protocol);
            handshake_base.extend_from_slice(&reserved_bytes);
            handshake_base.extend_from_slice(&info_hash);

            let mut handshake_message = handshake_base.clone();
            handshake_message.extend_from_slice(peer_id.as_bytes());

            let mut tcp_stream =
                TcpStream::connect(peer).expect(&format!("Connecting to peer {}", peer));
            tcp_stream
                .write(handshake_message.as_slice())
                .expect("Writing to peer");
            #[allow(unused_mut)]
            let mut buffer_response = &mut [0; 68];
            tcp_stream
                .read(buffer_response)
                .expect("Reading response from Peer");
            println!("Read ! Decoding..");

            let received_bytes = &buffer_response[0..48];

            if received_bytes.len() != handshake_base.len() {
                panic!(
                    "Array lengths don't match: {} vs {}",
                    received_bytes.len(),
                    handshake_base.len()
                );
            }
            let received_hash = &received_bytes[28..48];
            if received_hash != info_hash {
                panic!("Hashes don't match !");
            }

            println!("Peer ID: {}", hex::encode(&buffer_response[48..68]));
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
