#![allow(dead_code)]

#[derive(Debug)]
struct TcpHeader {
    source_port: u16,
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    flags: u8,
}

impl TcpHeader {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 20 {
            // TCP header is at least 20 bytes.
            return None;
        }

        Some(Self {
            source_port: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
            dest_port: u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
            seq_num: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            ack_num: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            flags: bytes[13],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_headers_from_bytes() {
        let raw_bytes: [u8; 20] = [
            0xC0, 0xA8, // Source Port: 49320 (0xC0A8 in hex)
            0x1F, 0x90, // Destination Port: 8080 (0x1F90 in hex)
            0x12, 0x34, 0x56, 0x78, // Sequence Number: 305419896 (0x12345678)
            0x87, 0x65, 0x43, 0x21, // Acknowledgment Number: 2271560481 (0x87654321)
            0x50, // Data Offset (4 bits): 5 (20 bytes), Reserved (3 bits): 0, Flags (9 bits): 0b00000000
            0x18, // Flags: SYN (0b00011000)
            0x00, 0xFF, // Window Size: 255
            0xF0, 0x0D, // Checksum: 61453 (0xF00D in hex)
            0x00, 0x00, // Urgent Pointer: 0
        ];

        let headers = TcpHeader::from_bytes(&raw_bytes).unwrap();

        assert_eq!(headers.source_port, 49320);
        assert_eq!(headers.dest_port, 8080);
        assert_eq!(headers.seq_num, 305419896);
        assert_eq!(headers.ack_num, 2271560481);
        assert_eq!(headers.flags, 0x18); // SYN + ACK
    }
}
