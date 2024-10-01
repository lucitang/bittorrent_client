use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};

struct PeersVisitor;

#[derive(Debug)]
pub struct Peers(pub Vec<SocketAddrV4>);

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Peers, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

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
        let peers: Vec<SocketAddrV4> = v
            .chunks_exact(6)
            .map(|chunk_6| {
                SocketAddrV4::new(
                    Ipv4Addr::new(chunk_6[0], chunk_6[1], chunk_6[2], chunk_6[3]),
                    u16::from_be_bytes([chunk_6[4], chunk_6[5]]),
                )
            })
            .collect();

        Ok(Peers(peers))
    }
}
