use crate::structs::handshake::Handshake;
use crate::structs::message::{Message, MessageType};
use std::io::{Read, Write};
use std::net::{SocketAddrV4, TcpStream};

fn connect(peer: SocketAddrV4) -> TcpStream {
    #[allow(unused_mut)]
    let mut tcp_stream = TcpStream::connect(peer).expect(&format!("Connecting to peer {}", peer));
    tcp_stream
}

/// Initialize a stream with a peer by sending a handshake message.
pub fn connect_tcp(info_hash: &[u8; 20], peer_id: &[u8; 20], peer: &SocketAddrV4) -> TcpStream {
    let mut tcp_stream = connect(*peer);

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

pub fn request_tcp(
    tcp_stream: &mut TcpStream,
    message_type: MessageType,
    payload: Vec<u8>,
    expected_payload_length: usize,
) -> Message {
    // Send message
    let message = Message::new(message_type as u8, payload);
    let message_bytes = message.to_bytes();
    tcp_stream
        .write(message_bytes.as_slice())
        .expect("Sending interested message");

    // Read the unchoke message
    let mut response: Vec<u8> = vec![0u8; 5 + expected_payload_length];
    tcp_stream
        .read(&mut response)
        .expect("Reading response from Peer");
    Message::from_bytes(&response)
}
