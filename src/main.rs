// use anyhow::{Context, Error};
// use bittorrent_starter_rust::structs::extension::Extension;
// use bittorrent_starter_rust::structs::peers::{Peer, PeerList};
// use bittorrent_starter_rust::structs::torrent::Torrent;
// use bittorrent_starter_rust::utils::decoder::decode_bencoded_value;
// use bittorrent_starter_rust::utils::files::write_file;
// use clap::Parser;
// use serde_bencode::from_bytes;
// use std::fs;
//
// use std::io::{self, Write};
// use bittorrent_starter_rust::structs::magnet::MagnetLink;
// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     loop {
//         // Display the menu options
//         println!("Choose a command to execute:");
//         println!("1. Decode");
//         println!("2. Info");
//         println!("3. Peers");
//         println!("4. Handshake");
//         println!("5. DownloadPiece");
//         println!("6. Download");
//         println!("7. MagnetParse");
//         println!("8. MagnetHandshake");
//         println!("9. MagnetInfo");
//         println!("10. MagnetDownloadPiece");
//         println!("11. MagnetDownload");
//         println!("0. Exit");
//
//         print!("Enter your choice: ");
//         io::stdout().flush()?; // Flush to show prompt
//         let mut choice = String::new();
//         io::stdin().read_line(&mut choice)?;
//         let choice = choice.trim();
//
//         match choice {
//             "1" => {
//                 let encoded_value = get_input("Enter the bencoded value to decode: ")?;
//                 let (decoded_value, _) = decode_bencoded_value(&encoded_value);
//                 println!("Decoded value: {}", decoded_value.to_string());
//             }
//             "2" => {
//                 let torrent_file = get_input("Enter the torrent file path: ")?;
//                 let file = fs::read(&torrent_file).context("Reading torrent file")?;
//                 let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
//                 println!("Tracker URL: {}", torrent.announce);
//                 println!("Length: {}", torrent.info.length);
//                 println!("Piece Length: {}", torrent.info.piece_length);
//                 println!("Info Hash: {}", hex::encode(torrent.info.get_hash()));
//                 println!("Piece Hashes:");
//                 for chunk in torrent.info.pieces.chunks(20) {
//                     println!("{}", hex::encode(chunk));
//                 }
//             }
//             "3" => {
//                 let torrent_file = get_input("Enter the torrent file path: ")?;
//                 let file = fs::read(&torrent_file).context("Reading torrent file")?;
//                 let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
//                 PeerList::get_peers(&torrent).await?;
//             }
//             "4" => {
//                 let torrent_file = get_input("Enter the torrent file path: ")?;
//                 let peer_address = get_input("Enter the peer address (IP:Port): ")?;
//                 let file = fs::read(&torrent_file).context("Reading torrent file")?;
//                 let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
//                 Peer::new(peer_address.parse()?, &torrent.info.get_hash()).await?;
//             }
//             "5" => {
//                 let torrent_file = get_input("Enter the torrent file path: ")?;
//                 let piece_index = get_input("Enter the piece index: ")?.parse::<i32>()?;
//                 let output = get_input("Enter the output file path: ")?;
//                 let file = fs::read(&torrent_file).context("Reading torrent file")?;
//                 let torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
//                 let mut available_peers = torrent.get_available_peers().await?;
//                 let data = available_peers[0]
//                     .download_piece(piece_index, torrent.get_piece_len(piece_index))
//                     .await?;
//                 write_file(&output, &data)?;
//                 println!("Piece saved to {}", output);
//             }
//             "6" => {
//                 let torrent_file = get_input("Enter the torrent file path: ")?;
//                 let output = get_input("Enter the output file path: ")?;
//                 let file = fs::read(&torrent_file).context("Reading torrent file")?;
//                 let mut torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
//                 let peers = torrent.get_available_peers().await?;
//                 if let Ok(pieces) = torrent.download_torrent(peers, false).await {
//                     let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
//                     write_file(&output, &data)?;
//                     println!("Torrent downloaded to {}", output);
//                 } else {
//                     eprintln!("Error downloading torrent");
//                 }
//             }
//             "7" => {
//                 let magnet_string = get_input("Enter the magnet link: ")?;
//                 let magnet_link: MagnetLink = magnet_string.parse()?;
//                 println!("Tracker URL: {}", magnet_link.tracker_url);
//                 println!("Info Hash: {}", hex::encode(magnet_link.info_hash))
//             }
//             "8" => {
//                 let magnet_string = get_input("Enter the magnet link: ")?;
//                 let magnet_link: MagnetLink = magnet_string.parse()?;
//                 let peers = PeerList::get_peers_from(&magnet_link).await?;
//                 if !peers.is_empty() {
//                     let mut peer = Peer::new(peers[0], &magnet_link.info_hash).await?;
//                     println!("Peer extensions: {:?}", peer.extensions);
//                     peer.get_pieces().await?;
//                     let ext = peer.send_ext_handshake().await?;
//                     println!("Peer Metadata Extension ID: {}", ext.inner.ut_metadata);
//                 }
//             }
//             "9" => {
//                 let magnet_string = get_input("Enter the magnet link: ")?;
//                 let magnet_link: MagnetLink = magnet_string.parse()?;
//                 println!("Tracker URL: {}", magnet_link.tracker_url);
//                 println!("Name: {:?}", magnet_link.name);
//                 let peers = PeerList::get_peers_from(&magnet_link).await?;
//                 if !peers.is_empty() {
//                     let mut peer = Peer::new(peers[0], &magnet_link.info_hash).await?;
//                     let ext = peer.send_ext_handshake().await?;
//                     let _info = peer.get_extension_info(&ext, &magnet_link).await?;
//                 }
//             }
//             "10" => {
//                 let magnet_string = get_input("Enter the magnet link: ")?;
//                 let magnet_link: MagnetLink = magnet_string.parse()?;
//                 let piece_index = get_input("Enter the piece index: ")?.parse::<i32>()?;
//                 let output = get_input("Enter the output file path: ")?;
//                 let peers = PeerList::get_peers_from(&magnet_link).await?;
//                 if !peers.is_empty() {
//                     let mut peer = Peer::new(peers[0], &magnet_link.info_hash).await?;
//                     let ext = peer.send_ext_handshake().await?;
//                     let info = peer.get_extension_info(&ext, &magnet_link).await?;
//                     let torrent = Torrent {
//                         announce: magnet_link.tracker_url,
//                         info,
//                     };
//                     let data = peer
//                         .download_piece(piece_index, torrent.get_piece_len(piece_index))
//                         .await?;
//                     write_file(&output, &data)?;
//                     println!("Piece saved to {}", output);
//                 }
//             }
//             "11" => {
//                 let magnet_string = get_input("Enter the magnet link: ")?;
//                 let magnet_link: MagnetLink = magnet_string.parse()?;
//                 let output = get_input("Enter the output file path: ")?;
//                 let peers = PeerList::get_peers_from(&magnet_link).await?;
//                 let mut available_peers: Vec<Peer> = vec![];
//                 let mut extension: Option<Extension> = None;
//
//                 for peer in peers {
//                     let mut peer = Peer::new(peer, &magnet_link.info_hash).await?;
//                     peer.get_pieces().await?;
//                     let ext = peer.send_ext_handshake().await?;
//                     if extension.is_none() {
//                         extension = Some(ext);
//                     }
//                     available_peers.push(peer);
//                 }
//
//                 if !available_peers.is_empty() {
//                     let info = available_peers[0]
//                         .get_extension_info(&extension.unwrap(), &magnet_link)
//                         .await?;
//                     let mut torrent = Torrent {
//                         announce: magnet_link.tracker_url,
//                         info,
//                     };
//                     if let Ok(pieces) = torrent.download_torrent(available_peers, true).await {
//                         let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
//                         write_file(&output, &data)?;
//                         println!("File saved to {}", output);
//                     } else {
//                         eprintln!("Error downloading torrent");
//                     }
//                 }
//             }
//             "0" => {
//                 println!("Exiting.");
//                 break;
//             }
//             _ => {
//                 println!("Invalid choice. Please try again.");
//             }
//         }
//     }
//     Ok(())
// }
//
// /// Helper function to get user input
// fn get_input(prompt: &str) -> Result<String, Error> {
//     print!("{}", prompt);
//     io::stdout().flush()?; // Ensure the prompt is displayed
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
//     Ok(input.trim().to_string())
// }

// use anyhow::{Context, Error};
// use bittorrent_starter_rust::structs::extension::Extension;
// use bittorrent_starter_rust::structs::peers::{Peer, PeerList};
// use bittorrent_starter_rust::structs::torrent::Torrent;
// use bittorrent_starter_rust::utils::files::write_file;
// use serde_bencode::from_bytes;
// use std::fs;
// use std::io::{self, Write};
// use bittorrent_starter_rust::structs::magnet::MagnetLink;
//
// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     loop {
//         // Display the menu options
//         println!("Choose a command to execute:");
//         println!("1. Download");
//         println!("2. MagnetDownload");
//         println!("0. Exit");
//
//         print!("Enter your choice: ");
//         io::stdout().flush()?; // Flush to show prompt
//         let mut choice = String::new();
//         io::stdin().read_line(&mut choice)?;
//         let choice = choice.trim();
//
//         match choice {
//             "1" => {
//                 let torrent_file = get_input("Enter the torrent file path: ")?;
//                 let output = get_input("Enter the output file path: ")?;
//                 let file = fs::read(&torrent_file).context("Reading torrent file")?;
//                 let mut torrent: Torrent = from_bytes(&file).context("Parsing file content")?;
//                 let peers = torrent.get_available_peers().await?;
//                 if let Ok(pieces) = torrent.download_torrent(peers, false).await {
//                     let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
//                     write_file(&output, &data)?;
//                     println!("Torrent downloaded to {}", output);
//                 } else {
//                     eprintln!("Error downloading torrent");
//                 }
//             }
//             "2" => {
//                 let magnet_string = get_input("Enter the magnet link: ")?;
//                 let magnet_link: MagnetLink = magnet_string.parse()?;
//                 let output = get_input("Enter the output file path: ")?;
//                 let peers = PeerList::get_peers_from(&magnet_link).await?;
//                 let mut available_peers: Vec<Peer> = vec![];
//                 let mut extension: Option<Extension> = None;
//
//                 for peer in peers {
//                     let mut peer = Peer::new(peer, &magnet_link.info_hash).await?;
//                     peer.get_pieces().await?;
//                     let ext = peer.send_ext_handshake().await?;
//                     if extension.is_none() {
//                         extension = Some(ext);
//                     }
//                     available_peers.push(peer);
//                 }
//
//                 if !available_peers.is_empty() {
//                     let info = available_peers[0]
//                         .get_extension_info(&extension.unwrap(), &magnet_link)
//                         .await?;
//                     let mut torrent = Torrent {
//                         announce: magnet_link.tracker_url,
//                         info,
//                     };
//                     if let Ok(pieces) = torrent.download_torrent(available_peers, true).await {
//                         let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
//                         write_file(&output, &data)?;
//                         println!("File saved to {}", output);
//                     } else {
//                         eprintln!("Error downloading torrent");
//                     }
//                 }
//             }
//             "0" => {
//                 println!("Exiting.");
//                 break;
//             }
//             _ => {
//                 println!("Invalid choice. Please try again.");
//             }
//         }
//     }
//     Ok(())
// }
//
// /// Helper function to get user input
// fn get_input(prompt: &str) -> Result<String, Error> {
//     print!("{}", prompt);
//     io::stdout().flush()?; // Ensure the prompt is displayed
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
//     Ok(input.trim().to_string())
// }


#[macro_use]
extern crate rocket;

use anyhow::{Context, Error};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{post, routes, tokio};
use bittorrent_starter_rust::structs::extension::Extension;
use bittorrent_starter_rust::structs::peers::{Peer, PeerList};
use bittorrent_starter_rust::structs::torrent::Torrent;
use bittorrent_starter_rust::utils::files::write_file;
use serde::Deserialize;
use serde_bencode::from_bytes;
use std::fs;

use bittorrent_starter_rust::structs::magnet::MagnetLink;

/// Request payload for Download (Torrent file)
#[derive(Deserialize)]
struct DownloadRequest {
    torrent_file_path: String,
    output_path: String,
}

/// Request payload for MagnetDownload
#[derive(Deserialize)]
struct MagnetDownloadRequest {
    magnet_link: String,
    magnet_output_path: String,
}

/// Torrent file download handler
#[post("/download", data = "<download_req>")]
async fn download_torrent(download_req: Json<DownloadRequest>) -> Result<Status, Json<String>> {
    let req = download_req.into_inner();

    let file = fs::read(&req.torrent_file_path)
        .context("Reading torrent file")
        .map_err(|e| Json(format!("Error: {}", e)))?;
    let mut torrent: Torrent = from_bytes(&file)
        .context("Parsing torrent file content")
        .map_err(|e| Json(format!("Error: {}", e)))?;

    let peers = torrent
        .get_available_peers()
        .await
        .map_err(|e| Json(format!("Error finding peers: {}", e)))?;

    if let Ok(pieces) = torrent.download_torrent(peers, false).await {
        let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
        write_file(&req.output_path, &data)
            .map_err(|e| Json(format!("Error saving file: {}", e)))?;
        Ok(Status::Ok)
    } else {
        Err(Json("Error downloading torrent".to_string()))
    }
}

/// Magnet link download handler
#[post("/magnet_download", data = "<magnet_req>")]
async fn magnet_download(magnet_req: Json<MagnetDownloadRequest>) -> Result<Status, Json<String>> {
    let req = magnet_req.into_inner();
    let magnet_link: MagnetLink = req
        .magnet_link
        .parse()
        .map_err(|e| Json(format!("Error parsing magnet link: {}", e)))?;

    let peers = PeerList::get_peers_from(&magnet_link)
        .await
        .map_err(|e| Json(format!("Error finding peers: {}", e)))?;
    let mut available_peers: Vec<Peer> = vec![];
    let mut extension: Option<Extension> = None;

    for peer in peers {
        let mut peer = Peer::new(peer, &magnet_link.info_hash)
            .await
            .map_err(|e| Json(format!("Error creating peer: {}", e)))?;
        peer.get_pieces()
            .await
            .map_err(|e| Json(format!("Error retrieving pieces: {}", e)))?;
        let ext = peer
            .send_ext_handshake()
            .await
            .map_err(|e| Json(format!("Error in handshake: {}", e)))?;
        if extension.is_none() {
            extension = Some(ext);
        }
        available_peers.push(peer);
    }

    if !available_peers.is_empty() {
        let info = available_peers[0]
            .get_extension_info(&extension.unwrap(), &magnet_link)
            .await
            .map_err(|e| Json(format!("Error retrieving torrent info: {}", e)))?;
        let mut torrent = Torrent {
            announce: magnet_link.tracker_url,
            info,
        };
        if let Ok(pieces) = torrent.download_torrent(available_peers, true).await {
            let data = pieces.into_iter().flatten().collect::<Vec<u8>>();
            write_file(&req.magnet_output_path, &data)
                .map_err(|e| Json(format!("Error saving file: {}", e)))?;
            Ok(Status::Ok)
        } else {
            Err(Json("Error downloading torrent".to_string()))
        }
    } else {
        Err(Json("No available peers found".to_string()))
    }
}

use rocket::fs::{FileServer, NamedFile};
use std::path::PathBuf;

#[rocket::get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("static/index.html").await.ok()
}

/// Rocket entry point
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![download_torrent, magnet_download, index])
        .mount("/static", FileServer::from("static"))
}
