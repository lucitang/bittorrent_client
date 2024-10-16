use crate::structs::message::{Message, MessageType};
use crate::structs::request::Request;
use crate::structs::torrent::Torrent;
use crate::utils::trackers;
use anyhow::Context;
use anyhow::Error;
use rand::random;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::cmp::min;
use std::fmt;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

/// Generate a peer id on 20 characters
/// Ex: 47001398037243657525
pub fn generate_peer_id() -> [u8; 20] {
    let mut peer_id: [u8; 20] = [0u8; 20];
    for i in 0..20 {
        peer_id[i] = (random::<u8>() % 10) + 48; // 48 is the ASCII code for '0'
    }
    peer_id
}

struct Handshake {
    pub protocol_byte: u8,
    pub protocol: [u8; 19],
    pub reserved_bytes: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Handshake {
        Handshake {
            peer_id,
            protocol_byte: 19,
            protocol: *b"BitTorrent protocol",
            info_hash,
            reserved_bytes: [0; 8],
        }
    }

    pub fn to_bytes(&self) -> [u8; 68] {
        let mut bytes = [0u8; 68];
        // Copying protocol byte, protocol, reserved bytes, info_hash, and peer_id in a more compact way
        bytes[0] = self.protocol_byte;
        bytes[1..20].copy_from_slice(&self.protocol);
        bytes[20..28].copy_from_slice(&self.reserved_bytes);
        bytes[28..48].copy_from_slice(&self.info_hash);
        bytes[48..68].copy_from_slice(&self.peer_id);

        bytes
    }
}

#[derive(Debug)]
pub struct PeerList(pub Vec<SocketAddrV4>);

struct PeersVisitor;

impl<'de> Deserialize<'de> for PeerList {
    fn deserialize<D>(deserializer: D) -> Result<PeerList, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = PeerList;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("6 bytes, the first 4 bytes are a peer's IP address and the last 2 are a peer's port number")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::invalid_length(v.len(), &self));
        }

        let peers: Vec<SocketAddrV4> = v
            .chunks_exact(6)
            .map(|chunk_6| {
                SocketAddrV4::new(
                    Ipv4Addr::new(chunk_6[0], chunk_6[1], chunk_6[2], chunk_6[3]),
                    u16::from_be_bytes([chunk_6[4], chunk_6[5]]),
                )
            })
            .collect();
        Ok(PeerList(peers))
    }
}

impl PeerList {
    pub async fn get_peers(torrent: &Torrent) -> Result<Vec<SocketAddrV4>, Error> {
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
        println!("Tracker Response: {:?}", tracker_response);
        for address in &tracker_response.peers.0 {
            println!("{}", address);
        }
        Ok(tracker_response.peers.0)
    }
}
#[derive(Debug, Clone)]
pub struct Peer {
    pub address: SocketAddrV4,
    pub stream: Arc<Mutex<TcpStream>>,
}

pub const MESSAGE_TYPES_WITHOUT_PAYLOAD: [MessageType; 4] = [
    MessageType::Choke,
    MessageType::Unchoke,
    MessageType::Interested,
    MessageType::NotInterested,
];
const BLOCK_SIZE: i32 = 16 * 1024; // = 16384 bytes

impl Peer {
    pub async fn new(address: SocketAddrV4, info_hash: &[u8; 20]) -> Result<Peer, Error> {
        let mut tcp_stream = TcpStream::connect(address)?;
        let peer_id = generate_peer_id();

        let handshake_bytes = Handshake::new(*info_hash, peer_id).to_bytes();
        tcp_stream.write(&handshake_bytes)?;

        #[allow(unused_mut)]
        let mut buffer_response = &mut [0; 68];
        tcp_stream.read(buffer_response)?;

        let received_bytes = &buffer_response[0..68];
        let received_hash = &received_bytes[28..48];
        if received_hash != info_hash {
            return Err(Error::msg("Hashes don't match !"));
        }

        println!("Peer ID: {}", hex::encode(&buffer_response[48..68]));
        Ok(Peer {
            address,
            stream: Arc::new(Mutex::new(tcp_stream)),
        })
    }

    pub async fn get_pieces(&mut self) -> Result<Vec<u8>, Error> {
        let message = &self.read().await?;
        println!("Response received: {:?}", message.message_type());
        if !matches!(message.message_type(), MessageType::Bitfield) {
            return Err(Error::msg("Expected bitfield message"));
        }

        let peer_pieces = message.payload.clone();
        println!("Bitfield pieces: {:?}", peer_pieces);
        Ok(peer_pieces)
        // TODO: implement the handling of the bitfield message.
    }

    pub async fn send_interest(&mut self) -> Result<(), Error> {
        // Send interested message
        let interested_message = Message::new(MessageType::Interested as u8, vec![]);
        self.send(interested_message).await?;

        // Read the response
        let message = self.read().await?;
        println!("Response received: {:?}", message.message_type()); // Should receive an Unchoke message.

        if !matches!(message.message_type(), MessageType::Unchoke) {
            return Err(Error::msg("Expected unchoke message"));
        }

        Ok(())
    }

    pub async fn download_piece(
        &mut self,
        piece_index: i32,
        piece_len: i32,
    ) -> Result<Vec<u8>, Error> {
        println!(
            "  - Downloading piece: piece_index: {}, piece_len: {}",
            piece_index, piece_len
        );
        let mut set = JoinSet::new();
        let mut piece_data = vec![0u8; piece_len as usize]; // Pre-allocate the vector for the piece data
        let spawn = |join_set: &mut JoinSet<_>,
                     mut peer: Peer,
                     piece_index: i32,
                     block_offset: i32,
                     block_length: i32| {
            join_set.spawn(async move {
                match peer
                    .download_block(piece_index, block_offset, block_length)
                    .await
                {
                    Ok(data) => (block_offset, data),
                    Err(e) => {
                        eprintln!("Error downloading block: {:?}", e);
                        (block_offset, vec![])
                    }
                }
            });
        };

        for offset in (0..piece_len).step_by(BLOCK_SIZE as usize) {
            let length = min(BLOCK_SIZE, piece_len - offset);
            spawn(&mut set, self.clone(), piece_index, offset, length);
        }

        while let Some(join_result) = set.join_next().await {
            let (block_offset, block_data) = join_result.context("Joining block")?;
            let block_offset = block_offset as usize;
            let block_data_len = block_data.len();
            piece_data[block_offset..block_offset + block_data_len].copy_from_slice(&block_data);
        }

        Ok(piece_data)
    }

    pub async fn download_block(
        &mut self,
        piece_index: i32,
        begin: i32,
        length: i32,
    ) -> Result<Vec<u8>, Error> {
        // println!(
        //     "  - Downloading block: piece_index: {}, begin: {}, length: {}",
        //     piece_index, begin, length
        // );
        let request = Request::new(piece_index, begin, length);
        let message = Message::new(MessageType::Request as u8, request.to_bytes());
        self.send(message).await?;
        let response = self.read().await?;
        let block_data = &response.payload[8..];

        Ok(block_data.to_vec())
    }

    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        let mut tcp_stream = self.stream.lock().await;
        tcp_stream.write(&message.to_bytes())?;
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Message, Error> {
        let mut tcp_stream = self.stream.lock().await;
        #[allow(unused_mut)]
        let mut buf = &mut [0; 4];
        tcp_stream
            .read_exact(buf)
            .context("Reading message length (Are Piece index/len correct ?")?;
        let prefix = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

        let mut buf = vec![0; 1];
        tcp_stream
            .read_exact(&mut buf)
            .context("Reading message id")?;
        let message_id = buf[0];

        let message_type = MessageType::from_byte(message_id);
        if MESSAGE_TYPES_WITHOUT_PAYLOAD.contains(&message_type) {
            return Ok(Message::new(message_id, vec![]));
        }

        let mut buf = vec![0; prefix - 1]; // -1 for message_id
        tcp_stream
            .read_exact(&mut buf)
            .context("Reading message payload")?;
        Ok(Message::new(message_id, buf))
    }
}
