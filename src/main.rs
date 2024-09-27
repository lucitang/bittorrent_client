use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json;
use serde_json::{Map, Number, Value};
use sha1::{Digest, Sha1};
use std::{env, fs};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct Torrent {
    // URL to a "tracker", which is a central server that keeps track of peers participating in the sharing of a torrent.
    announce: String,
    // TorrentInfo Dictionnary
    info: TorrentInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
struct TorrentInfo {
    // Size of the file in bytes, for single-file torrents
    length: Number,

    // Suggested name to save the file / directory as
    name: String,

    // Number of bytes in each piece
    #[serde(rename = "piece length")]
    piece_length: usize,

    // Concatenated SHA-1 hashes of each piece
    pieces: ByteBuf,
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    // Usage: your_bittorrent.sh decode "<encoded_value>"
    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let (decoded_value, _) = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let torrent_file = &args[2];
        // println!("Torrent File: {torrent_file}");
        let file = fs::read(torrent_file).context("Reading torrent file")?;
        let torrent: Torrent = serde_bencode::from_bytes(&file).context("Parsing file content")?;
        // println!("Deserialized torrent: {:#?}", torrent);
        println!("Tracker URL: {}", torrent.announce);
        println!("Length: {}", torrent.info.length);

        let code = serde_bencode::to_bytes(&torrent.info)?;
        let mut hasher = Sha1::new();
        hasher.update(&code.as_slice());
        let digest = hasher.finalize();
        println!("Info Hash: {digest:x}");
    } else {
        println!("unknown command: {}", args[1]);
    }

    Ok(())
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (Value, &str) {
    let (tag, mut rest) = encoded_value.split_at(1);
    match tag.chars().next() {
        // A dictionary is encoded as d<key1><value1>...<keyN><valueN>e. <key1>, <value1> etc. correspond to the bencoded keys & values.
        // The keys are sorted in lexicographical order and must be strings.
        // For example, {"hello": 52, "foo":"bar"} would be encoded as: d3:foo3:bar5:helloi52ee (note that the keys were reordered).
        Some('d') => {
            let mut dic: Map<String, Value> = Map::new();

            while !rest.is_empty() && !rest.starts_with('e') {
                let (key, remains) = decode_bencoded_value(rest);
                if let Some(key) = key.as_str() {
                    let (value, remains) = decode_bencoded_value(remains);
                    dic.insert(key.to_string(), value);
                    rest = remains;
                } else {
                    panic!("Key '{key}' should be a string");
                }
            }

            return (dic.into(), &rest[1..]);
        }

        // Lists are encoded as l<bencoded_elements>e.
        // For example, ["hello", 52] would be encoded as l5:helloi52ee.
        // Note that there are no separators between the elements
        Some('l') => {
            let mut values: Vec<Value> = Vec::new();

            while !rest.is_empty() && !rest.starts_with('e') {
                let (v, remaining) = decode_bencoded_value(rest);
                values.push(v);
                rest = remaining;
            }

            return (values.into(), &rest[1..]); // omit the first 'e'.
        }

        // Integers are encoded as i<number>e.
        // For example, 52 is encoded as i52e and -52 is encoded as i-52e.
        Some('i') => {
            if let Some((value, rest)) = rest
                .split_once('e')
                .and_then(|(value, rest)| Some((value.parse::<i64>().ok()?, rest)))
            {
                return (value.into(), rest);
            }
        }

        // If encoded_value starts with a digit, it's a number
        // Example: "5:hello" -> "hello"
        Some('0'..='9') => {
            if let Some((len, rest)) = rest.split_once(':').and_then(|(chars, rest)| {
                Some(((tag.to_owned() + chars).parse::<usize>().ok()?, rest))
            }) {
                return (rest[..len].to_string().into(), &rest[len..]);
            }
        }

        _ => {
            println!("Unmatched encoded value: {encoded_value}")
        }
    }

    panic!("Unhandled encoded value: {}", encoded_value)
}
