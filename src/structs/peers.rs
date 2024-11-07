use crate::structs::extension::{
    Extension, ExtensionMessageType, InnerDictionnary, MetadataInfo, MetadataPayload,
};
use crate::structs::handshake::Handshake;
use crate::structs::magnet::MagnetLink;
use crate::structs::message::{Message, MessageType};
use crate::structs::request::Request;
use crate::structs::torrent::{Torrent, TorrentInfo};
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
use tokio::sync::{Mutex, OwnedSemaphorePermit};
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
    /// Get the list of peers from a magnet link
    pub async fn get_peers_from(magnet_link: &MagnetLink) -> Result<Vec<SocketAddrV4>, Error> {
        let encoded_info = magnet_link
            .info_hash
            .iter()
            .map(|b| format!("%{:02x}", b))
            .collect::<String>();

        let query_params = trackers::QueryParams {
            peer_id: generate_peer_id().iter().map(|b| *b as char).collect(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            // This is an arbitrary value, we don't know the total length of the file yet.
            left: 999,
            compact: 1,
        };

        let tracker_response =
            trackers::get_tracker_info(&magnet_link.tracker_url, query_params, encoded_info)
                .await
                .context("Getting tracker info")?;
        // println!("Tracker Response: {:?}", tracker_response);
        let peers = tracker_response.peers.unwrap_or(PeerList(vec![]));
        Ok(peers.0)
    }

    /// Get the list of peers from a torrent file
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
        let peers = tracker_response.peers.unwrap_or(PeerList(vec![]));
        for address in &peers.0 {
            println!("{}", address);
        }
        Ok(peers.0)
    }
}
#[derive(Debug, Clone)]
pub struct Peer {
    pub address: SocketAddrV4,
    pub stream: Arc<Mutex<TcpStream>>,
    pub peer_id: String,
    pub extensions: Vec<u8>,
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

        let req_handshake = Handshake::new(*info_hash, peer_id);
        tcp_stream.write(&req_handshake.to_bytes())?;

        #[allow(unused_mut)]
        let mut buffer_response = &mut [0; 68];
        tcp_stream.read(buffer_response)?;

        let handshake_response = Handshake::from_bytes(buffer_response);
        if handshake_response.info_hash != *info_hash {
            return Err(Error::msg("Hashes don't match !"));
        }

        let mut extensions = vec![];
        if handshake_response.reserved_bytes != [0u8; 8] {
            extensions.push(handshake_response.reserved_bytes[5]);
        }

        println!("Peer ID: {}", handshake_response.peer_id_string());
        Ok(Peer {
            address,
            stream: Arc::new(Mutex::new(tcp_stream)),
            peer_id: handshake_response.peer_id_string(),
            extensions,
        })
    }

    pub async fn get_pieces(&mut self) -> Result<Vec<u8>, Error> {
        let message = &self.read().await?;
        // println!("Response received: {:?}", message.message_type());
        if !matches!(message.message_type(), MessageType::Bitfield) {
            return Err(Error::msg("Expected bitfield message"));
        }

        let peer_pieces = message.payload.clone();
        // println!("Bitfield pieces: {:?}", peer_pieces);
        Ok(peer_pieces)
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

        let semaphore = Arc::new(tokio::sync::Semaphore::new(5));

        let spawn = |join_set: &mut JoinSet<_>,
                     mut peer: Peer,
                     piece_index: i32,
                     block_offset: i32,
                     block_length: i32,
                     permit: OwnedSemaphorePermit| {
            join_set.spawn(async move {
                match peer
                    .download_block(piece_index, block_offset, block_length)
                    .await
                {
                    Ok(data) => {
                        drop(permit);
                        (block_offset, data)
                    }
                    Err(e) => {
                        eprintln!("Error downloading block: {:?}", e);
                        (block_offset, vec![])
                    }
                }
            });
        };
        for offset in (0..piece_len).step_by(BLOCK_SIZE as usize) {
            let length = min(BLOCK_SIZE, piece_len - offset);
            let permit = semaphore.clone().acquire_owned().await?;
            spawn(&mut set, self.clone(), piece_index, offset, length, permit);
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
        //     "    - Downloading block: piece_index: {}, begin: {}, length: {}",
        //     piece_index, begin, length
        // );
        let request = Request::new(piece_index, begin, length);
        let message = Message::new(MessageType::Request as u8, request.to_bytes());
        self.send(message).await?;
        let response = self.read().await?;
        let block_data = &response.payload[8..];

        Ok(block_data.to_vec())
    }

    /// Extension messages follow the standard BitTorrent message format:
    ///
    /// message length prefix (4 bytes)
    /// message id (1 byte)
    /// This will be 20 for all messages implemented by extensions
    /// payload (variable size)
    /// The payload will be structured as follows:
    ///
    /// extension message id (1 byte)
    /// This will be 0 for the extension handshake
    /// bencoded dictionary (variable size)
    /// This will contain a key "m" with another dictionary as its value.
    /// The inner dictionary maps supported extension names to their corresponding message IDs./
    ///
    /// For example, the inner dictionary contents might be {"ut_metadata": 1, "ut_pex": 2},
    /// indicating that your peer supports the "utmetadata" and "utpex" extensions with IDs 1 and 2 respectively.
    pub async fn send_ext_handshake(&mut self) -> Result<Extension, Error> {
        // Extension support message
        let extension = Extension {
            inner: InnerDictionnary { ut_metadata: 1 },
            metadata_size: 0,
        };

        // Message ID is 0 for the extension handshake
        let mut bytes = vec![0];
        bytes.extend(serde_bencode::to_bytes(&extension)?);
        let message = Message::new(20, bytes);
        self.send(message).await?;

        // Read peer extension message
        let response = self
            .read()
            .await
            .context("Reading extension message response")?;
        let (message_id, bencoded_dict) = response.payload.split_at(1);
        assert_eq!(message_id[0], 0);
        let ext: Extension = serde_bencode::from_bytes(bencoded_dict)?;
        Ok(ext)
    }

    pub async fn get_extension_info(
        &mut self,
        extension: &Extension,
        magnet_link: &MagnetLink,
    ) -> Result<TorrentInfo, Error> {
        println!(
            "Peer Metadata Extension ID: {}",
            extension.inner.ut_metadata
        );
        println!("Peer Metadata Size: {}", extension.metadata_size);
        let (_meta, torrent_info) = self
            .request_metadata(extension.inner.ut_metadata, 0)
            .await?;

        println!("Length: {}", torrent_info.length);
        println!("Info Hash: {}", hex::encode(&magnet_link.info_hash));

        println!("Piece Length: {}", torrent_info.piece_length);
        for chunk in torrent_info.pieces.chunks(20) {
            println!("{:}", hex::encode(chunk));
        }
        // Verify hash is valid
        assert_eq!(torrent_info.get_hash(), magnet_link.info_hash);

        Ok(torrent_info)
    }

    pub async fn request_metadata(
        &mut self,
        extensions_id: u8,
        piece_index: u8,
    ) -> Result<(MetadataInfo, TorrentInfo), Error> {
        let payload = MetadataPayload {
            piece: piece_index,
            msg_type: ExtensionMessageType::Request as u8,
        };

        // Message ID is =the extension ID
        let mut bytes = vec![extensions_id];
        bytes.extend(serde_bencode::to_bytes(&payload)?);
        let message = Message::new(20, bytes);
        self.send(message).await?;

        // Read peer extension message
        let response = self
            .read()
            .await
            .context("Reading metadata message response")?;

        let (message_id, remains) = response.payload.split_at(1);

        println!("MessageID {:?}", message_id);
        println!(
            "Received payload {:?}",
            remains.iter().map(|v| *v as char).collect::<String>()
        );
        println!("Received message ID {}", message_id[0]);
        // TODO: fix this
        // assert_eq!(message_id[0], extensions_id);
        let metadata_info: MetadataInfo =
            serde_bencode::from_bytes(remains).context("Decoding metadata info")?;
        assert_eq!(metadata_info.msg_type, ExtensionMessageType::Data as u8);
        let meta_size = serde_bencode::to_bytes(&metadata_info)
            .context("Encoding metadata info")?
            .len();

        println!("Received Metadata {:?}", metadata_info);
        let torrent_info: TorrentInfo =
            serde_bencode::from_bytes(&remains[meta_size..]).context("Decoding torrent info")?;
        println!("Received TorrentInfo {:?}", torrent_info);

        Ok((metadata_info, torrent_info))
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
