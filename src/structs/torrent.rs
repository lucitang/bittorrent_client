use crate::structs::peers::{Peer, PeerList};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use tokio::task::JoinSet;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Torrent {
    /// URL to a "tracker", which is a central server that keeps track of peers participating in the sharing of a torrent.
    pub announce: String,

    /// This maps to a dictionary, with keys described below.
    pub info: TorrentInfo,
}

#[allow(dead_code)]
impl Torrent {
    pub fn check_piece_hash(&self, piece_index: i32, pieces_data: &Vec<u8>) -> bool {
        let curr_piece = self
            .info
            .pieces
            .chunks(20)
            .nth(piece_index as usize)
            .expect("Getting piece");
        let mut hasher = Sha1::new();
        hasher.update(&pieces_data);
        let digest = hasher.finalize();
        digest.as_slice() == curr_piece
    }

    pub fn info_hash(&self) -> [u8; 20] {
        let code = serde_bencode::to_bytes(&self.info).expect("Bencoding the info section");
        let mut hasher = Sha1::new();
        hasher.update(&code.as_slice());
        hasher
            .finalize()
            .try_into()
            .expect("Converting digest to [u8; 20] array")
    }

    pub fn info_hash_string(&self) -> String {
        self.info_hash()
            .iter()
            .map(|b| format!("%{:02x}", b))
            .collect::<String>()
    }

    pub async fn get_available_peers(&self) -> Result<Vec<Peer>, Error> {
        // Step 1: get the peer list from the Tacker
        let addresses = PeerList::get_peers(self).await?;
        let mut available_peers: Vec<Peer> = vec![];

        // Step 2: Get the available peers
        for address in addresses {
            let mut peer = Peer::new(address, &self.info_hash()).await?;
            // TODO: improve when the bitfield is implemented
            peer.get_pieces().await?;
            // Add if the peer can send pieces.
            match peer.send_interest().await {
                Ok(..) => available_peers.push(peer),
                Err(..) => {}
            }
        }
        Ok(available_peers)
    }

    pub fn get_piece_len(&self, piece_index: i32) -> i32 {
        self.info
            .piece_length
            .min(self.info.length - piece_index * self.info.piece_length)
    }

    pub async fn download_torrent(&mut self) -> Result<Vec<Vec<u8>>, Error> {
        let peers = self.get_available_peers().await?;
        let piece_count = self.info.pieces.chunks(20).len();
        let mut pieces_result: Vec<Vec<u8>> = vec![vec![]; piece_count];
        let pending_pieces: Vec<PendingPiece> = (0..piece_count as i32)
            .map(|piece_index| PendingPiece {
                piece_index,
                peers: peers.clone(),
            })
            .collect();

        let spawn = |join_set: &mut JoinSet<_>, pending_piece: PendingPiece| {
            let piece_len = self.get_piece_len(pending_piece.piece_index);
            join_set.spawn(async move {
                let mut piece_data = vec![];
                for mut peer in pending_piece.peers {
                    if let Ok(data) = peer
                        .download_piece(pending_piece.piece_index, piece_len)
                        .await
                    {
                        piece_data = data;
                        break;
                    }
                }
                (pending_piece.piece_index, piece_data)
            });
        };

        let mut join_set = JoinSet::new();
        for pending_piece in pending_pieces {
            spawn(&mut join_set, pending_piece);
        }

        while let Some(result) = join_set.join_next().await {
            if let Ok((index, data)) = result {
                if data.len() == 0 {
                    eprintln!("Error downloading piece. Index: {}", index);
                } else {
                    pieces_result[index as usize] = data;
                }
            }
        }

        Ok(pieces_result)
    }
}

struct PendingPiece {
    piece_index: i32,
    peers: Vec<Peer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct TorrentInfo {
    /// The length of the file, in bytes.
    /// For single-file torrents only (length is only present when the download represents a single file)
    pub length: i32,

    /// The name key maps to a UTF-8 encoded string which is the suggested name to save the file (or directory) as. It is purely advisory
    /// @link: https://www.bittorrent.org/beps/bep_0003.html#info-dictionary
    pub name: String,

    /// Number of bytes in each piece
    ///
    /// **piece length** maps to the number of bytes in each piece the file is split into.
    /// For the purposes of transfer, files are split into fixed-size pieces
    /// which are all the same length except for possibly the last one which may be truncated.
    ///
    /// **piece length** is almost always a power of two,
    /// most commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M as default).
    /// @link: https://www.bittorrent.org/beps/bep_0003.html#info-dictionary
    #[serde(rename = "piece length")]
    pub piece_length: i32,

    /// Concatenated SHA-1 hashes of each piece
    /// **pieces** maps to a string whose length is a multiple of 20.
    /// It is to be subdivided into strings of length 20,
    /// each of which is the SHA1 hash of the piece at the corresponding index.
    ///
    /// Every 20 bytes of this string is the SHA1 hash (or `&[u8]` chunk of length `20`) of a piece.
    pub pieces: ByteBuf,
}

impl TorrentInfo {
    pub fn len(&self) -> i32 {
        // For single files, this will do, but for multiple files, we need to iterate over the files and sum their lengths
        self.length
    }
}
