use crate::structs::message::{Message, MessageType};
use crate::structs::peers::MESSAGE_TYPES_WITHOUT_PAYLOAD;
use anyhow::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::io::AsyncReadExt;

pub struct Request {
    pub piece_index: i32,
    pub begin: i32,
    pub length: i32,
}

impl Request {
    pub fn new(piece_index: i32, begin: i32, length: i32) -> Request {
        Request {
            piece_index,
            begin,
            length,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend_from_slice(&self.piece_index.to_be_bytes());
        bytes.extend_from_slice(&self.begin.to_be_bytes());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes
    }

    pub fn send(stream: &Option<TcpStream>, message: Message) -> Result<(), Error> {
        stream.write(&message.to_bytes()).expect("Writing to peer");
        Ok(())
    }

    pub fn read(tcp_stream: &Option<TcpStream>) -> Result<Message, Error> {
        #[allow(unused_mut)]
        let mut buf = &mut [0; 4];
        #[allow(unused_mut)]
        tcp_stream.read_exact(buf).expect("Reading message length");
        let prefix = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

        let mut buf = vec![0; 1];
        tcp_stream.read_exact(&mut buf).expect("Reading message id");
        let message_id = buf[0];

        let message_type = MessageType::from_byte(message_id);
        if MESSAGE_TYPES_WITHOUT_PAYLOAD.contains(&message_type) {
            return Ok(Message::new(message_id, vec![]));
        }

        let mut buf = vec![0; prefix - 1]; // -1 for message_id
        tcp_stream
            .read_exact(&mut buf)
            .expect("Reading message payload");
        Ok(Message::new(message_id, buf))
    }
}
