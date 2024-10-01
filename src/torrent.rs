use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Torrent {
    /// URL to a "tracker", which is a central server that keeps track of peers participating in the sharing of a torrent.
    pub announce: String,

    /// This maps to a dictionary, with keys described below.
    pub info: TorrentInfo,
}

impl Torrent {
    pub fn info_hash(&self) -> [u8; 20] {
        let code = serde_bencode::to_bytes(&self.info).expect("Bencoding the info section");
        let mut hasher = Sha1::new();
        hasher.update(&code.as_slice());
        hasher
            .finalize()
            .try_into()
            .expect("Converting digest to [u8; 20] array")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct TorrentInfo {
    /// The length of the file, in bytes.
    /// For single-file torrents only (length is only present when the download represents a single file)
    pub length: usize,

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
    pub piece_length: usize,

    /// Concatenated SHA-1 hashes of each piece
    /// **pieces** maps to a string whose length is a multiple of 20.
    /// It is to be subdivided into strings of length 20,
    /// each of which is the SHA1 hash of the piece at the corresponding index.
    ///
    /// Every 20 bytes of this string is the SHA1 hash (or `&[u8]` chunk of length `20`) of a piece.
    pub pieces: ByteBuf,
}
