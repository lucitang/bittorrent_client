use crate::structs::message::{Message, MessageType};
use crate::structs::peers::{generate_peer_id, Peer};
use crate::structs::request::Request;
use crate::structs::torrent::Torrent;
use anyhow::Error;
use std::cmp::min;

pub struct Worker {
    peers: Vec<Peer>,
    peer_id: [u8; 20],
}

impl Worker {
    pub fn new(peers: Vec<Peer>) -> Self {
        let peer_id = generate_peer_id();
        Self { peers, peer_id }
    }

    pub fn download_torrent(&mut self, torrent: &Torrent) -> Result<Vec<Vec<u8>>, Error> {
        let piece_count = torrent.info.pieces.chunks(20).len();
        let info_hash = torrent.info_hash();
        let mut pieces: Vec<Vec<u8>> = vec![vec![]; piece_count];
        let mut queue: Vec<usize> = (0..piece_count).collect();

        for peer in &mut self.peers {
            if queue.is_empty() {
                println!("All pieces downloaded");
                break;
            }
            // Make sur the peer is ready to receive requests
            if let Err(..) = Worker::check_readiness(&info_hash, peer, &self.peer_id) {
                println!("Peer {} is not ready, trying next one", peer.address);
                continue;
            }

            let mut next_queue: Vec<usize> = Vec::new();
            while !queue.is_empty() {
                let piece_index = queue.pop().expect("Queue is not empty");
                println!(
                    "Downloading piece {} from peer {}",
                    piece_index, peer.address
                );
                if let Ok(piece_data) = Worker::download_piece(piece_index as i32, &torrent, peer) {
                    println!("Piece {} verified and downloaded successfully", piece_index);
                    pieces[piece_index] = piece_data;
                } else {
                    next_queue.push(piece_index);
                }
            }
            queue = next_queue;
        }

        if queue.is_empty() {
            println!("All pieces downloaded successfully");
        } else {
            return Err(Error::msg("Failed to download all pieces"));
        }

        Ok(pieces)
    }

    // fn set_next_available_peer(&mut self) -> Result<(), Error> {
    //     if self.peer_index < self.peers.len() - 1 {
    //         self.peer_index += 1;
    //         return Ok(());
    //     }
    //
    //     Err(Error::msg("No more peers available"))
    // }

    pub fn check_readiness(
        info_hash: &[u8; 20],
        peer: &mut Peer,
        peer_id: &[u8; 20],
    ) -> Result<(), Error> {
        #[allow(unused_mut)]
        peer.handshake(&info_hash, &peer_id)?;
        println!("–––––––––––––––––––––––––––––––––––––");
        // Expect a bitfield message
        let message = peer.read();
        println!("Response received: {:?}", message.message_type());
        if !matches!(message.message_type(), MessageType::Bitfield) {
            return Err(Error::msg("Expected bitfield message"));
        }

        // Send interested message
        let interested_message = Message::new(MessageType::Interested as u8, vec![]);
        peer.send(interested_message)?;
        // Read the response
        let message = peer.read();
        println!("Response received: {:?}", message.message_type()); // Should receive an Unchoke message.
        if !matches!(message.message_type(), MessageType::Unchoke) {
            return Err(Error::msg("Expected unchoke message"));
        }

        Ok(())
    }

    pub fn download_piece(
        piece_index: i32,
        torrent: &Torrent,
        peer: &mut Peer,
    ) -> Result<Vec<u8>, Error> {
        let mut piece_data = Vec::new();
        let piece_length: i32 = min(
            torrent.info.length - piece_index * torrent.info.piece_length,
            torrent.info.piece_length,
        );
        println!(
            "Torrent Length: {} | Piece ({}) length: {}",
            torrent.info.length, piece_index, piece_length
        );

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

            peer.send(msg)?;
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
            return Err(Error::msg("Piece hash doesn't match"));
        }

        if piece_data.len() as i32 != piece_length {
            return Err(Error::msg("Piece length doesn't match"));
        }
        Ok(piece_data)
    }
}
const BLOCK_SIZE: i32 = 16 * 1024; // = 16384 bytes
