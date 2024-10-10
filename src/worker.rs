use crate::structs::message::{Message, MessageType};
use crate::structs::peers::{generate_peer_id, Peer};
use crate::structs::request::Request;
use crate::structs::torrent::Torrent;
use anyhow::Error;
use std::cmp::min;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

pub struct Worker {
    peers: Vec<Peer>,
    peer_id: [u8; 20],
}

impl Worker {
    pub fn new(peers: Vec<Peer>) -> Self {
        let peer_id = generate_peer_id();
        Self { peers, peer_id }
    }

    pub async fn download_torrent(&mut self, torrent: &Torrent) -> Result<Vec<Vec<u8>>, Error> {
        let piece_count = torrent.info.pieces.chunks(20).len();
        let info_hash = torrent.info_hash();
        let mut pieces: Vec<Vec<u8>> = vec![vec![]; piece_count];
        let mut pieces_queue: Vec<usize> = (0..piece_count).collect();

        for peer in &mut self.peers {
            if pieces_queue.is_empty() {
                println!("All pieces downloaded");
                break;
            }
            // Make sur the peer is ready to receive requests
            if let Err(..) = Worker::check_readiness(&info_hash, peer, &self.peer_id) {
                println!("Peer {} is not ready, trying next one", peer.address);
                continue;
            }

            let mut next_queue: Vec<usize> = Vec::new();
            while !pieces_queue.is_empty() {
                let piece_index = pieces_queue.pop().expect("Queue is not empty");
                println!(
                    "Downloading piece {} from peer {}",
                    piece_index, peer.address
                );
                if let Ok(piece_data) =
                    Worker::download_piece(piece_index as i32, &torrent, peer).await
                {
                    println!("Piece {} verified and downloaded successfully", piece_index);
                    pieces[piece_index] = piece_data;
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

        let message = Request::read(&peer.stream).expect("Reading response");
        println!("Response received: {:?}", message.message_type());
        if !matches!(message.message_type(), MessageType::Bitfield) {
            return Err(Error::msg("Expected bitfield message"));
        }

        // Send interested message
        let interested_message = Message::new(MessageType::Interested as u8, vec![]);
        Request::send(&peer.stream, interested_message)?;
        // Read the response
        let message = Request::read(&peer.stream).expect("Reading response");
        println!("Response received: {:?}", message.message_type()); // Should receive an Unchoke message.
        if !matches!(message.message_type(), MessageType::Unchoke) {
            return Err(Error::msg("Expected unchoke message"));
        }

        Ok(())
    }

    pub async fn download_piece(
        piece_index: i32,
        torrent: &Torrent,
        peer: &mut Peer,
    ) -> Result<Vec<u8>, Error> {
        let mut join_handles: Vec<JoinHandle<Result<(), Error>>> = Vec::new();
        // To limit the number of concurrent requests to a peer
        let mut semaphore = Arc::new(Semaphore::new(5));
        let m_stream = Mutex::new(&peer.stream);
        let mut shared_stream = Arc::new(m_stream);
        let piece_length: i32 = min(
            torrent.info.length - piece_index * torrent.info.piece_length,
            torrent.info.piece_length,
        );
        let mut piece_data: Vec<u8> = Vec::with_capacity(piece_length as usize);
        println!(
            "Torrent Length: {} | Piece ({}) length: {}",
            torrent.info.length, piece_index, piece_length
        );
        let a_piece_data = Arc::new(Mutex::new(piece_data));

        let block_count = if piece_length % BLOCK_SIZE == 0 {
            piece_length / BLOCK_SIZE
        } else {
            (piece_length / BLOCK_SIZE) + 1
        };
        println!("Block count {}", block_count);

        for i in 0..block_count {
            let stream = shared_stream.clone(); // Cloning Arc, not the stream itself
            let c_piece_data = a_piece_data.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            join_handles.push(tokio::spawn(async move {
                println!("–––––––––––––––––––––––––––––––––––––");
                // Calculate the length of the block to request given the previous block size and the piece length
                let block_length: i32 = min(BLOCK_SIZE, piece_length - i * BLOCK_SIZE);

                // Prepare the payload for the request message
                let request: Request = Request::new(piece_index, i * BLOCK_SIZE, block_length);
                let msg = Message::new(MessageType::Request as u8, request.to_bytes());

                // Lock the stream without cloning it
                let mut tcp_stream = stream.lock().unwrap(); // Locking the Arc<Mutex<TcpStream>>

                Request::send(&mut *tcp_stream, msg).expect("Sending request");
                let response = Request::read(&mut *tcp_stream).expect("Reading response");
                drop(tcp_stream);
                println!("- Received {:?} message", response.message_type(),);
                if matches!(response.message_type(), MessageType::Piece) {
                    let mut l_piece_data = c_piece_data.lock().unwrap();
                    // The 2 first bytes of the payload are the index and begin fields. The rest is the block data
                    l_piece_data.extend_from_slice(&response.payload[8..]);
                    println!(
                        "- Remaining bytes: {}",
                        piece_length - l_piece_data.len() as i32
                    );
                    drop(l_piece_data);
                }
                drop(permit);
                Ok(())
            }));
        }

        for handle in join_handles {
            handle.await?;
        }

        println!("–––––––––––––––––––––––––––––––––––––");

        let r_piece_data = (*a_piece_data).lock().unwrap();
        // Check the integrity of the piece with it's hash value from the torrent file.
        if !torrent.check_piece_hash(piece_index, &r_piece_data) {
            return Err(Error::msg("Piece hash doesn't match"));
        }

        if r_piece_data.len() != piece_length as usize {
            return Err(Error::msg("Piece length doesn't match"));
        }
        Ok(r_piece_data.to_vec())
    }
}
const BLOCK_SIZE: i32 = 16 * 1024; // = 16384 bytes
