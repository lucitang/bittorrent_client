use anyhow::{Context, Error};
use bittorrent_starter_rust::cli::{Cli, Commands};
use bittorrent_starter_rust::decoder::decode_bencoded_value;
use bittorrent_starter_rust::files::write_file;
use bittorrent_starter_rust::structs::peers::{generate_peer_id, Peer, PeerList};
use bittorrent_starter_rust::structs::torrent::Torrent;
use bittorrent_starter_rust::worker::Worker;
use clap::Parser;
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
            PeerList::get_peers(&torrent).await?;
        }
        Commands::Handshake {
            torrent_file,
            peer_address,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let info_hash = torrent.info_hash();
            let peer_id = generate_peer_id();
            let mut peer = Peer::from(&peer_address);
            peer.handshake(&info_hash, &peer_id);
        }
        Commands::Download {
            torrent_file,
            output,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let peers = PeerList::get_peers(&torrent).await?;
            let mut worker = Worker::new(peers);
            let pieces: Vec<u8> = worker.download_torrent(&torrent);
            write_file(&output, &pieces);
            println!("File saved to {}", output);
        }
    };

    Ok(())
}
