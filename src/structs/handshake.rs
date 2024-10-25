pub struct Handshake {
    pub protocol_byte: u8,
    pub protocol: [u8; 19],
    pub reserved_bytes: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

// Should have an extension so that the reserved bytes can be set to '00 00 00 00 00 10 00 00'
const RESERVED_BYTES: [u8; 8] = [0, 0, 0, 0, 0, 0x10, 0, 0];

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Handshake {
        Handshake {
            peer_id,
            protocol_byte: 19,
            protocol: *b"BitTorrent protocol",
            info_hash,
            reserved_bytes: RESERVED_BYTES,
        }
    }

    pub fn to_bytes(&self) -> [u8; 68] {
        let mut bytes = [0u8; 68];
        // Copying protocol byte, protocol, reserved bytes, info_hash, and peer_id in a more compact way
        bytes[0] = self.protocol_byte;
        bytes[1..20].copy_from_slice(&self.protocol);
        bytes[20..28].copy_from_slice(&self.reserved_bytes);
        bytes[28..48].copy_from_slice(&self.info_hash);
        bytes[48..68].copy_from_slice(&self.peer_id);

        bytes
    }
}
