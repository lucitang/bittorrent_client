#[derive(Debug)]
pub struct Message {
    /// message length prefix (4 bytes)
    prefix: [u8; 4],

    /// message id (1 byte)
    message_id: u8,

    /// All non-keepalive messages start with a single byte which gives their type.
    /// The possible values are:
    ///
    /// 0 - choke
    /// 1 - unchoke
    /// 2 - interested
    /// 3 - not interested
    /// 4 - have
    /// 5 - bitfield
    /// 6 - request
    /// 7 - piece
    /// 8 - cancel
    ///
    /// 'choke', 'unchoke', 'interested', and 'not interested' have no payload.
    pub payload: Vec<u8>,
}

#[repr(u8)]
#[derive(Debug)]
pub enum MessageType {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl Message {
    pub fn new(message_id: u8, payload: Vec<u8>) -> Message {
        Message {
            prefix: (1 + payload.len() as u32).to_be_bytes(),
            message_id,
            payload,
        }
    }
    pub fn new_from_type(message_type: MessageType, payload: Vec<u8>) -> Message {
        Message::new(message_type as u8, payload)
    }

    pub fn set_length(&mut self, length: u32) {
        self.prefix = length.to_be_bytes();
    }

    pub fn from_bytes(bytes: &[u8]) -> Message {
        if bytes.len() < 5 {
            panic!("Message is too short");
        }
        let prefix = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let message_id = bytes[4];
        let payload = bytes[5..].to_vec();

        Message {
            prefix,
            message_id,
            payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.prefix);
        bytes.push(self.message_id);
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    pub fn message_type(&self) -> MessageType {
        match self.message_id {
            0 => MessageType::Choke,
            1 => MessageType::Unchoke,
            2 => MessageType::Interested,
            3 => MessageType::NotInterested,
            4 => MessageType::Have,
            5 => MessageType::Bitfield,
            6 => MessageType::Request,
            7 => MessageType::Piece,
            8 => MessageType::Cancel,
            _ => panic!("Unknown message type: {}", self.message_id),
        }
    }
}
