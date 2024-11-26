#![allow(dead_code)]

#[derive(Debug)]
struct TcpHeader {
    source_port: u16,
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    flags: u8,
    window_size: u16,
    checksum: u16,
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
            window_size: u16::from_be_bytes(bytes[14..16].try_into().unwrap()),
            checksum: u16::from_be_bytes(bytes[16..18].try_into().unwrap()),
        })
    }

    fn to_be_bytes(&self) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        bytes[0..2].copy_from_slice(&self.source_port.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.dest_port.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.seq_num.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.ack_num.to_be_bytes());
        bytes[12] = (5 << 4) | 0; // Reserved = 0, data offset 5.
        bytes[13] = self.flags;
        bytes[14..16].copy_from_slice(&self.window_size.to_be_bytes());
        bytes[16..18].copy_from_slice(&self.checksum.to_be_bytes());
        bytes[18..20].copy_from_slice(&0u16.to_be_bytes());

        bytes
    }

    fn build_packet(&self, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&self.to_be_bytes());

        packet.extend_from_slice(payload);

        packet
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
        assert_eq!(headers.window_size, 255);
        assert_eq!(headers.checksum, 61453);
    }

    #[test]
    fn test_tcp_headers_to_bytes() {
        let header = TcpHeader {
            source_port: 49320,
            dest_port: 8080,
            seq_num: 305419896,
            ack_num: 2271560481,
            flags: 0x18, // SYN + ACK
            window_size: 255,
            checksum: 61453, // 0xF00D
        };
        let raw_bytes = header.to_be_bytes();

        assert_eq!(
            raw_bytes,
            [
                0xC0, 0xA8, // Source Port: 49320 (0xC0A8 in hex)
                0x1F, 0x90, // Destination Port: 8080 (0x1F90 in hex)
                0x12, 0x34, 0x56, 0x78, // Sequence Number: 305419896 (0x12345678)
                0x87, 0x65, 0x43, 0x21, // Acknowledgment Number: 2271560481 (0x87654321)
                0x50, // Data Offset (4 bits): 5 (20 bytes), Reserved (3 bits): 0, Flags (9 bits): 0b00000000
                0x18, // Flags: SYN (0b00011000)
                0x00, 0xFF, // Window Size: 255
                0xF0, 0x0D, // Checksum: 61453 (0xF00D in hex)
                0x00, 0x00, // Urgent Pointer: 0
            ]
        )
    }

    #[test]
    fn test_headers_build_packet_payload() {
        let header = TcpHeader {
            source_port: 49320,
            dest_port: 8080,
            seq_num: 305419896,
            ack_num: 2271560481,
            flags: 0x18, // SYN + ACK
            window_size: 255,
            checksum: 61453, // 0xF00D
        };
        // let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let payload = b"Hello, TCP!";
        let packet = header.build_packet(payload);

        assert_eq!(&packet[20..], payload); // Ensure payload is added
        assert_eq!(packet.len(), 20 + payload.len()); // Total packet size
    }
}
