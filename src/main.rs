use anyhow::{Context, Error};
use bittorrent_starter_rust::cli::{Cli, Commands};
use bittorrent_starter_rust::decoder::decode_bencoded_value;
use bittorrent_starter_rust::messaging::{connect_tcp, request_tcp};
use bittorrent_starter_rust::structs::message::{Message, MessageType};
use bittorrent_starter_rust::structs::torrent::Torrent;
use bittorrent_starter_rust::trackers;
use clap::Parser;
use rand::random;
use serde_bencode::from_bytes;
use std::fs;
use std::io::Read;
use std::net::SocketAddrV4;

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
            get_peers(&torrent).await?;
        }
        Commands::Handshake { torrent_file, peer } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let info_hash = torrent.info_hash();
            let peer_id = generate_peer_id();
            connect_tcp(&info_hash, &peer_id, &peer);
        }
        Commands::DownloadPiece {
            torrent_file,
            output: _,
            piece_index,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let info_hash = torrent.info_hash();
            let peer_id = generate_peer_id();
            let peers = get_peers(&torrent).await?;
            let mut tcp_stream = connect_tcp(&info_hash, &peer_id, &peers[0]);

            println!("–––––––––––––––––––––––––––––––––––––");
            let mut message: Vec<u8> = vec![0u8; 1024];
            tcp_stream
                .read(&mut message)
                .expect("Reading response from Peer");
            let message = Message::from_bytes(&message);
            println!("Response received: {:?}", message.message_type());

            if !matches!(message.message_type(), MessageType::Bitfield) {
                panic!("Expected bitfield message");
            }

            println!("–––––––––––––––––––––––––––––––––––––");
            // Send interested message
            let unchoke_response =
                request_tcp(&mut tcp_stream, MessageType::Interested, vec![0u8; 0], 0);
            println!("Response received: {:?}", unchoke_response.message_type()); // Should receive an Unchoke message.

            // the zero-based byte offset within the piece
            let mut begin: u32 = 0;
            let block_size: u32 = 16 * 1024; // 16 * 1024 bytes (16 kiB)

            println!("–––––––––––––––––––––––––––––––––––––");
            println!("Requesting piece {}", piece_index);
            println!("Piece length: {}", torrent.info.piece_length);

            // Break the torrent pieces into blocks of 16 kiB (16 * 1024 bytes) and send a request message for each block
            while (begin as usize) < torrent.info.piece_length {
                println!("Requesting chunk {}", begin);
                let mut payload: Vec<u8> = vec![piece_index];
                payload.extend_from_slice(&begin.to_be_bytes());
                // Calculate the length of the block to request given the previous block size and the piece length
                let length = std::cmp::min(block_size, torrent.info.piece_length as u32 - begin);
                payload.extend_from_slice(&length.to_be_bytes());

                let msg = Message::new_from_type(MessageType::Request, payload);
                println!("- Sending payload {:?}", msg.to_bytes());
                begin += length;

                let response = request_tcp(
                    &mut tcp_stream,
                    msg.message_type(),
                    msg.to_bytes(),
                    length as usize,
                );
                println!(
                    "- Chunk {} response received: {:?} | Payload size {}",
                    begin,
                    response.message_type(),
                    response.payload.len()
                );
                println!(
                    "- Remaming bytes: {}",
                    torrent.info.piece_length - begin as usize
                );
            }
            //
            // for chunks in torrent.info.pieces.chunks(20) {
            //     let block: Vec<u8> = vec![0u8; 16 * 1024];
            //
            //     println!("Chunk size: {}", chunks.len());
            //     let length: u8 = 0;
            //     let payload: Vec<u8> = vec![piece_index, begin, length];
            //     let msg = Message::new_from_type(MessageType::Request, payload);
            //     println!("Sending payload {:?}", msg.to_bytes());
            //     begin += 1;
            //
            //     let response = request_tcp(&mut tcp_stream, msg.message_type(), msg.to_bytes());
            //     println!(
            //         "Chunk {} response received: {:?}",
            //         begin,
            //         message.message_type()
            //     );
            // }
        }
    };

    Ok(())
}

async fn get_peers(torrent: &Torrent) -> Result<Vec<SocketAddrV4>, Error> {
    let encoded_info = torrent.info_hash_string();

    let query_params = trackers::QueryParams {
        peer_id: generate_peer_id().iter().map(|b| *b as char).collect(),
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
    for peer in &tracker_response.peers.0 {
        println!("{}", peer);
    }
    Ok(tracker_response.peers.0)
}

/// Generate a peer id on 20 characters
/// Ex: 47001398037243657525
fn generate_peer_id() -> [u8; 20] {
    let mut peer_id: [u8; 20] = [0u8; 20];
    for i in 0..20 {
        peer_id[i] = (random::<u8>() % 10) + 48; // 48 is the ASCII code for '0'
    }
    peer_id
}
