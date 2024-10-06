use crate::structs::message::{Message, MessageType};
use crate::structs::torrent::Torrent;
use crate::trackers;
use anyhow::Context;
use rand::random;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

/// Generate a peer id on 20 characters
/// Ex: 47001398037243657525
pub fn generate_peer_id() -> [u8; 20] {
    let mut peer_id: [u8; 20] = [0u8; 20];
    for i in 0..20 {
        peer_id[i] = (random::<u8>() % 10) + 48; // 48 is the ASCII code for '0'
    }
    peer_id
}

struct PeersVisitor;

#[derive(Debug)]
pub struct PeerList(pub Vec<Peer>);

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
        E: Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::invalid_length(v.len(), &self));
        }
        let peers: Vec<Peer> = v
            .chunks_exact(6)
            .map(|chunk_6| Peer::from(chunk_6))
            .collect();
        Ok(PeerList(peers))
    }
}

impl PeerList {
    pub async fn get_peers(torrent: &Torrent) -> Result<Vec<Peer>, anyhow::Error> {
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
            println!("{}", peer.address);
        }
        Ok(tracker_response.peers.0)
    }
}

#[derive(Debug)]
pub struct Peer {
    pub address: SocketAddrV4,
    stream: Option<TcpStream>,
}
impl From<&[u8]> for Peer {
    fn from(bytes: &[u8]) -> Peer {
        if bytes.len() != 6 {
            panic!("Invalid peer length");
        }
        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let port = u16::from_be_bytes([bytes[4], bytes[5]]);
        Peer {
            address: SocketAddrV4::new(ip, port),
            stream: None,
        }
    }
}

impl From<&SocketAddrV4> for Peer {
    fn from(address: &SocketAddrV4) -> Peer {
        Peer {
            address: *address,
            stream: None,
        }
    }
}

pub const MESSAGE_TYPES_WITHOUT_PAYLOAD: [MessageType; 4] = [
    MessageType::Choke,
    MessageType::Unchoke,
    MessageType::Interested,
    MessageType::NotInterested,
];

impl Peer {
    pub fn new(address: SocketAddrV4) -> Peer {
        Peer {
            address,
            stream: None,
        }
    }

    pub fn handshake(&mut self, info_hash: &[u8; 20], peer_id: &[u8; 20]) {
        let mut tcp_stream =
            TcpStream::connect(self.address).expect(&format!("Connecting to peer {:?}", self));

        let handshake_bytes = Handshake::new(*info_hash, *peer_id).to_bytes();
        tcp_stream.write(&handshake_bytes).expect("Writing to peer");

        #[allow(unused_mut)]
        let mut buffer_response = &mut [0; 68];
        tcp_stream
            .read(buffer_response)
            .expect("Reading Handshake response from Peer");

        let received_bytes = &buffer_response[0..68];
        let received_hash = &received_bytes[28..48];
        if received_hash != info_hash {
            panic!("Hashes don't match !");
        }

        println!("Peer ID: {}", hex::encode(&buffer_response[48..68]));
        self.stream = Some(tcp_stream);
    }

    pub fn send(&mut self, message: Message) {
        let tcp_stream = self.stream.as_mut().expect("No stream available");
        tcp_stream
            .write(&message.to_bytes())
            .expect("Writing to peer");
    }

    pub fn read(&mut self) -> Message {
        let tcp_stream = self.stream.as_mut().expect("No stream available");
        #[allow(unused_mut)]
        let mut buf = &mut [0; 4];
        tcp_stream.read_exact(buf).expect("Reading message length");
        let prefix = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

        let mut buf = vec![0; 1];
        tcp_stream.read_exact(&mut buf).expect("Reading message id");
        let message_id = buf[0];

        let message_type = MessageType::from_byte(message_id);
        if MESSAGE_TYPES_WITHOUT_PAYLOAD.contains(&message_type) {
            return Message::new(message_id, vec![]);
        }

        let mut buf = vec![0; prefix - 1]; // -1 for message_id
        tcp_stream
            .read_exact(&mut buf)
            .expect("Reading message payload");
        Message::new(message_id, buf)
    }
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
