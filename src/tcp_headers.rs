#![allow(dead_code)]

use std::net::Ipv4Addr;

#[derive(Debug)]
struct TcpHeader {
    source_port: u16,
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    flags: u8,
    window_size: u16,
    /// The checksum field is the 16-bit ones' complement of the ones'
    /// complement sum of all 16-bit words in the header and text.
    /// TODO: checksum is not used explicitly, but it calculted when needed:
    ///         Do we need to store it or calculate it on the fly?
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

    fn to_bytes(&self) -> [u8; 20] {
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

    fn to_be_bytes(&self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) -> [u8; 20] {
        let checksum = self.calculate_checksum(src_ip, dst_ip, payload);
        let mut raw_bytes = self.to_bytes();
        raw_bytes[16..18].copy_from_slice(&checksum.to_be_bytes());
        raw_bytes
    }

    fn build_packet(&self, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&self.to_bytes());

        packet.extend_from_slice(payload);

        packet
    }

    /// Returns: 16-bit ones' complement of the ones' complement sum of all
    /// 16-bit words in the header and text.
    ///
    /// The checksum computation needs to ensure the 16-bit alignment of the
    /// data being summed. If a segment contains an odd number of header and
    /// text octets, alignment can be achieved by padding the last octet with
    /// zeros on its right to form a 16-bit word for checksum purposes.
    ///
    /// The checksum also covers a pseudo-header conceptually prefixed to the
    /// TCP header. The pseudo-header is 96 bits for IPv4 (12 bytes, 4 per row).
    ///
    ///   +--------+--------+--------+--------+
    ///   |           Source Address          |
    ///   +--------+--------+--------+--------+
    ///   |         Destination Address       |
    ///   +--------+--------+--------+--------+
    ///   |  zero  |PTCL (6)|    TCP Length   |
    ///   +--------+--------+--------+--------+
    fn calculate_checksum(&self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) -> u16 {
        // The checksum itself is, according to the spec, is 16-bit long.
        // we use only the first two bytes of the u32 to do all summations.
        let mut sum = 0u32;

        // Pseudo-header: src IP (4 bytes)
        sum += u16::from_be_bytes(src_ip.octets()[0..2].try_into().unwrap()) as u32;
        sum += u16::from_be_bytes(src_ip.octets()[2..4].try_into().unwrap()) as u32;

        // Pseudo-header: dst IP (4 bytes)
        sum += u16::from_be_bytes(dst_ip.octets()[0..2].try_into().unwrap()) as u32;
        sum += u16::from_be_bytes(dst_ip.octets()[2..4].try_into().unwrap()) as u32;

        // Pseudo-header: Reserved (0), Protocol number: (6), TCP Length
        sum += 0x06_u32; // Protocol = 6 for TCP
        let tcp_length = (self.to_bytes().len() + payload.len()) as u16;
        sum += tcp_length as u32;

        // TCP headers
        for chunk in self.to_bytes().chunks(2) {
            sum += u16::from_be_bytes(chunk.try_into().unwrap()) as u32;
        }

        // Payload
        for chunk in payload.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes(chunk.try_into().unwrap()) as u32;
            } else {
                sum += (chunk[0] as u16) as u32;
            }
        }

        // Fold 32-bit sum into 16-bit.
        //
        // If the addition of the high and low 16 bits produces any carry-out,
        // (i.e. the new sum exceeds 16 bits) the process is repetead till no
        // carry-out.
        // perseving the mathematical correctness of one's complement by
        // adding any carry-out back the the lower 16 bits.
        while (sum >> 16) > 0 {
            // `sum && 0xFFF` extract the low 16 bits of sum.
            // `sum >> 16` extracts the high 16 bits of sum,
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    const TEST_HEADERS: TcpHeader = TcpHeader {
        source_port: 49320,
        dest_port: 8080,
        seq_num: 305419896,
        ack_num: 2271560481,
        flags: 0x18, // SYN + ACK
        window_size: 255,
        checksum: 61453, // 0xF00D
    };

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
        let raw_bytes = TEST_HEADERS.to_bytes();

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
        let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let packet = TEST_HEADERS.build_packet(payload);

        assert_eq!(&packet[20..], payload); // Ensure payload is added
        assert_eq!(packet.len(), 20 + payload.len()); // Total packet size
    }

    #[test]
    fn test_tcp_checksum_calculation() {
        let src_ip = Ipv4Addr::new(192, 168, 1, 1);
        let dst_ip = Ipv4Addr::new(192, 168, 1, 2);
        let payload = b"Hello, TCP!";

        let checksum = TEST_HEADERS.calculate_checksum(src_ip, dst_ip, payload);
        assert_ne!(checksum, 0); // Ensure checksum is non-zero
    }
}
