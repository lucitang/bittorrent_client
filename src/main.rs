use anyhow::{Context, Error};
use bittorrent_starter_rust::cli::{Cli, Commands};
use bittorrent_starter_rust::decoder::decode_bencoded_value;
use bittorrent_starter_rust::files::write_file;
use bittorrent_starter_rust::structs::message::{Message, MessageType};
use bittorrent_starter_rust::structs::peers::{generate_peer_id, Peer, PeerList};
use bittorrent_starter_rust::structs::request::Request;
use bittorrent_starter_rust::structs::torrent::Torrent;
use clap::Parser;
use serde_bencode::from_bytes;
use std::cmp::min;
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
        Commands::DownloadPiece {
            torrent_file,
            output,
            piece_index,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let info_hash = torrent.info_hash();
            let peer_id = generate_peer_id();
            let peers = PeerList::get_peers(&torrent).await?;
            let mut peer = Peer::from(&peers[1].address);
            peer.handshake(&info_hash, &peer_id);

            println!("–––––––––––––––––––––––––––––––––––––");
            // Expect a bitfield message
            let message = peer.read();
            println!("Response received: {:?}", message.message_type());
            if !matches!(message.message_type(), MessageType::Bitfield) {
                panic!("Expected bitfield message");
            }

            // Send interested message
            let interested_message = Message::new(MessageType::Interested as u8, vec![]);
            peer.send(interested_message);
            // Read the response
            let message = peer.read();
            println!("Response received: {:?}", message.message_type()); // Should receive an Unchoke message.

            // TODO: Check if the message is an Unchoke message

            println!("Requesting piece {}", piece_index);

            let mut piece_data = Vec::new();
            println!("Torrent Length: {}", torrent.info.length);
            let piece_length: i32 = min(
                torrent.info.length - piece_index * torrent.info.piece_length,
                torrent.info.piece_length,
            );
            println!("Piece length: {}", piece_length);
            // Break the torrent pieces into blocks of 16 kiB (16 * 1024 bytes) and send a request message for each block
            while (piece_data.len() as i32) < piece_length {
                println!("–––––––––––––––––––––––––––––––––––––");

                // Calculate the length of the block to request given the previous block size and the piece length
                let length: i32 = min(BLOCK_SIZE, piece_length as i32 - piece_data.len() as i32);

                println!(
                    "Requesting chunk from {} | block of size {}",
                    piece_data.len(),
                    length
                );

                // Prepare the payload for the request message
                let request: Request = Request::new(piece_index, piece_data.len() as i32, length);
                let msg = Message::new(MessageType::Request as u8, request.to_bytes());

                peer.send(msg);
                let response = peer.read();
                println!("- Received {:?} message", response.message_type(),);
                if matches!(response.message_type(), MessageType::Piece) {
                    // The 2 first bytes of the payload are the index and begin fields. The rest is the block data
                    piece_data.extend_from_slice(&response.payload[8..]);
                    println!(
                        "- Remaining bytes: {}",
                        piece_length - piece_data.len() as i32
                    );
                }
            }
            println!("–––––––––––––––––––––––––––––––––––––");

            // Check the integrity of the piece with it's hash value from the torrent file.
            if !torrent.check_piece_hash(piece_index, &piece_data) {
                panic!("Piece hash doesn't match");
            }

            if piece_data.len() as i32 != piece_length {
                panic!("Failed to download piece");
            }
            println!("Piece verified and downloaded successfully");
            write_file(&output, &piece_data);
            println!("Piece saved to {}", output);
        }
    };

    Ok(())
}

const BLOCK_SIZE: i32 = 16 * 1024; // = 16384 bytes
