use anyhow::{Context, Error};
use bittorrent_starter_rust::cli::{Cli, Commands};
use bittorrent_starter_rust::structs::peers::{Peer, PeerList};
use bittorrent_starter_rust::structs::torrent::Torrent;
use bittorrent_starter_rust::utils::decoder::decode_bencoded_value;
use bittorrent_starter_rust::utils::files::write_file;
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
            let torrent_hash = torrent.info.get_hash();
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
            let info_hash = torrent.info.get_hash();
            Peer::new(peer_address, &info_hash).await?;
        }
        Commands::DownloadPiece {
            piece_index,
            torrent_file,
            output,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            let mut available_peers: Vec<Peer> = torrent.get_available_peers().await?;

            println!("Torrent length: {}", torrent.info.length);
            let piece_len = torrent.get_piece_len(piece_index);
            let mut file_data = vec![0u8; piece_len as usize]; // for the purpose of this test, this needs to be the piece size
            let data = available_peers[1]
                .download_piece(piece_index, piece_len)
                .await?;

            if data.len() != piece_len as usize {
                eprintln!("Error downloading piece: invalid length");
                return Ok(());
            }
            file_data[..piece_len as usize].copy_from_slice(&data);
            write_file(&output, &file_data)?;
        }
        Commands::Download {
            torrent_file,
            output,
        } => {
            let file = fs::read(torrent_file).context("Reading torrent file")?;
            let mut torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
            if let Ok(pieces) = torrent.download_torrent().await {
                let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
                if data.len() != torrent.info.length as usize {
                    eprintln!("Error downloading torrent: invalid length");
                    return Ok(());
                }
                write_file(&output, &data)?;
                println!("File saved to {}", output);
            } else {
                eprintln!("Error downloading torrent");
            }
        }
        Commands::MagnetParse { magnet_link } => {
            println!("Tracker URL: {}", magnet_link.tracker_url);
            println!("Info Hash: {}", hex::encode(magnet_link.info_hash));
        }
        Commands::MagnetHandshake { magnet_link } => {
            // println!("Tracker URL: {}", magnet_link.tracker_url);
            // println!("Info Hash: {}", hex::encode(&magnet_link.info_hash));
            let peers = PeerList::get_peers_from(&magnet_link).await?;
            if peers.len() > 0 {
                let mut peer = Peer::new(peers[0], &magnet_link.info_hash).await?;
                println!("Peer extensions: {:?}", peer.extensions);
                peer.get_pieces().await?;
                let ext = peer.get_extension().await?;
                println!("Peer Metadata Extension ID: {}", ext.inner.ut_metadata);
            }
        }
        Commands::MagnetInfo { magnet_link } => {
            println!("Tracker URL: {}", magnet_link.tracker_url);
            println!("Name: {:?}", &magnet_link.name);
            let peers = PeerList::get_peers_from(&magnet_link).await?;
            if peers.len() > 0 {
                let mut peer = Peer::new(peers[0], &magnet_link.info_hash).await?;
                println!("Peer extensions: {:?}", peer.extensions);
                peer.get_pieces().await?;
                let ext = peer.get_extension().await?;
                let _info = peer.get_extension_info(&ext, &magnet_link).await?;
            }
        }
        Commands::MagnetDownloadPiece {
            piece_index,
            magnet_link,
            output,
        } => {
            println!("Tracker URL: {}", magnet_link.tracker_url);
            println!("Name: {:?}", &magnet_link.name);
            println!("Piece Index: {:?}", piece_index);
            let peers = PeerList::get_peers_from(&magnet_link).await?;

            if peers.len() > 0 {
                let mut peer = Peer::new(peers[2], &magnet_link.info_hash).await?;
                println!("Peer extensions: {:?}", peer.extensions);
                peer.get_pieces().await?;
                let ext = peer.get_extension().await?;
                let info = peer.get_extension_info(&ext, &magnet_link).await?;
                // let mut torrent = Torrent {
                //     info,
                //     announce: magnet_link.tracker_url,
                // };

                peer.send_interest().await?;
                let data = peer.download_piece(piece_index, info.piece_length).await?;

                if data.len() != info.piece_length as usize {
                    eprintln!("Error downloading piece: invalid length");
                    return Ok(());
                }
                let mut file_data = vec![0u8; info.piece_length as usize]; // for the purpose of this test, this needs to be the piece size
                file_data[..info.piece_length as usize].copy_from_slice(&data);
                write_file(&output, &file_data)?;

                // if let Ok(pieces) = torrent.download_torrent().await {
                //     let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
                //     if data.len() != torrent.info.length as usize {
                //         eprintln!("Error downloading torrent: invalid length");
                //         return Ok(());
                //     }
                //     write_file(&output, &data)?;
                //     println!("File saved to {}", output);
                // } else {
                //     eprintln!("Error downloading torrent");
                // }
            }
        }
    };

    Ok(())
}
