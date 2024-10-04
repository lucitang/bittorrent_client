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
}
