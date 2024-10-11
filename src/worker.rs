use crate::structs::peers::Peer;
use crate::structs::torrent::Torrent;
use anyhow::Error;
use std::cmp::min;

pub struct Worker {
    peers: Vec<Peer>,
}

impl Worker {
    pub fn new(peers: Vec<Peer>) -> Self {
        Self { peers }
    }

    pub async fn download_torrent(&mut self, torrent: &Torrent) -> Result<Vec<Vec<u8>>, Error> {
        let piece_count = torrent.info.pieces.chunks(20).len();
        let mut pieces: Vec<Vec<u8>> = vec![vec![]; piece_count];
        let mut pieces_queue: Vec<i32> = (0..piece_count as i32).collect();

        for peer in &mut self.peers {
            if pieces_queue.is_empty() {
                println!("All pieces downloaded");
                break;
            }

            let mut next_queue: Vec<i32> = Vec::new();
            while !pieces_queue.is_empty() {
                let piece_index: i32 = pieces_queue.pop().expect("Queue is not empty");
                let piece_length: i32 = torrent
                    .info
                    .piece_length
                    .min(torrent.info.length - piece_index * torrent.info.piece_length);
                println!(
                    "Downloading piece {} from peer {}",
                    piece_index, peer.address
                );
                if let Ok(piece_data) = peer.download_piece(piece_index, piece_length).await {
                    println!("Piece {} verified and downloaded successfully", piece_index);
                    pieces[piece_index as usize] = piece_data;
                } else {
                    next_queue.push(piece_index);
                }
            }
            pieces_queue = next_queue;
        }

        if pieces_queue.is_empty() {
            println!("All pieces downloaded successfully");
        } else {
            return Err(Error::msg("Failed to download all pieces"));
        }

        Ok(pieces)
    }
}
