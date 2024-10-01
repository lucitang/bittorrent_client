use crate::structs::handshake::Handshake;
use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpStream};

fn connect(peer: SocketAddrV4) -> TcpStream {
    #[allow(unused_mut)]
    let mut tcp_stream = TcpStream::connect(peer).expect(&format!("Connecting to peer {}", peer));
    tcp_stream
}
pub fn stream_handshake(info_hash: &[u8; 20], peer_id: &[u8; 20], peer: SocketAddrV4) -> TcpStream {
    // let protocol_byte: &u8 = &19;
    // let protocol: &[u8; 19] = b"BitTorrent protocol";
    // let reserved_bytes: [u8; 8] = [0; 8];
    //
    // let mut handshake_base: Vec<u8> = Vec::new();
    // handshake_base.push(*protocol_byte);
    // handshake_base.extend_from_slice(protocol);
    // handshake_base.extend_from_slice(&reserved_bytes);
    // handshake_base.extend_from_slice(info_hash);
    //
    // let mut handshake_message = handshake_base.clone();
    // handshake_message.extend_from_slice(peer_id.as_bytes());

    let mut tcp_stream = connect(peer);

    let handshake_bytes = Handshake::new(*info_hash, *peer_id).to_bytes();
    tcp_stream.write(&handshake_bytes).expect("Writing to peer");
    #[allow(unused_mut)]
    let mut buffer_response = &mut [0; 68];
    tcp_stream
        .read(buffer_response)
        .expect("Reading response from Peer");

    let received_bytes = &buffer_response[0..68];

    if received_bytes.len() != handshake_bytes.len() {
        panic!(
            "Array lengths don't match: {} vs {}",
            received_bytes.len(),
            &handshake_bytes.len()
        );
    }
    let received_hash = &received_bytes[28..48];
    if received_hash != info_hash {
        panic!("Hashes don't match !");
    }

    println!("Peer ID: {}", hex::encode(&buffer_response[48..68]));
    tcp_stream
}
