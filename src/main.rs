use anyhow::{Context, Error};
use bittorrent_starter_rust::cli::{Cli, Commands};
use bittorrent_starter_rust::decoder::decode_bencoded_value;
use bittorrent_starter_rust::files::write_file;
use bittorrent_starter_rust::structs::peers::{Peer, PeerList};
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
            Peer::new(peer_address, &info_hash).await?;
        }
        Commands::Download {
            torrent_file,
            output,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let info_hash = torrent.info_hash();

            // Step 1: get the peer list
            let addresses = PeerList::get_peers(&torrent).await?;

            // Step 2: Connect to the peers
            let mut available_peers: Vec<Peer> = vec![];

            // Step 3: Get the available peers
            for address in addresses {
                let mut peer = Peer::new(address, &info_hash).await?;
                // TODO: improve when the bitfield is implemented
                peer.get_pieces().await?;
                // Add if the peer can send pieces.
                match peer.send_interest().await {
                    Ok(..) => available_peers.push(peer),
                    Err(..) => {}
                }
            }

            let mut worker = Worker::new(available_peers);
            if let Ok(pieces) = worker.download_torrent(&torrent).await {
                let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
                write_file(&output, &data);
                println!("File saved to {}", output);
            }
        }
    };

    Ok(())
}
