#![allow(dead_code)]

use crate::flags::TcpFlags;
use core::panic;
use std::{fmt, net::Ipv4Addr};

#[derive(Debug)]
pub struct Tcp {
    pub source_port: u16,
    pub dest_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub flags: TcpFlags,
    pub window_size: u16,
    pub checksum: u16,
}

impl fmt::Display for Tcp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = format!(
            r#"TCP Header:
    Source Port: {}
    Destination Port: {}
    Sequence Number: {}
    Acknowledge Number: {}
    Flags: {}"#,
            self.source_port, self.dest_port, self.seq_num, self.ack_num, self.flags
        );
        write!(f, "{}", output)
    }
}

impl Tcp {
    pub fn to_bytes(&self) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        bytes[0..2].copy_from_slice(&self.source_port.to_be_bytes());
        bytes[2..4].copy_from_slice(&self.dest_port.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.seq_num.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.ack_num.to_be_bytes());
        bytes[12] = (5 << 4) | 0; // Reserved = 0, data offset 5.
        bytes[13] = self.flags.bits();
        bytes[14..16].copy_from_slice(&self.window_size.to_be_bytes());
        bytes[16..18].copy_from_slice(&self.checksum.to_be_bytes());
        bytes[18..20].copy_from_slice(&0u16.to_be_bytes());
        bytes
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

    pub fn build_packet(&self, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&self.to_bytes());
        packet.extend_from_slice(payload);

        packet
    }

    pub fn parse_packet(bytes: &[u8]) -> (Tcp, Option<String>) {
        let tcp = Tcp::try_from(bytes).unwrap();
        let payload = if bytes.len() > 20 {
            Some(String::from_utf8_lossy(&bytes[20..]).into_owned())
        } else {
            None
        };

        (tcp, payload)
    }
}

/// Converts a slice of bytes into a `Tcp` instance.
///
/// # Parameters
/// - `bytes`: A slice of bytes representing a TCP header. Must be at least 20 bytes long.
///
/// # Panics
/// This function will panic if the provided `bytes` slice is less than 20 bytes long.
///
/// # Notes
/// - The function assumes the input byte slice follows the TCP header structure.
/// - The `flags` field is parsed into a `TcpFlags` instance, ensuring valid flag combinations.
///
/// # Example
/// ```
/// use harbinger::tcp::Tcp;
/// let raw_bytes: [u8; 20] = [
///     0xC0, 0xA8, 0x1F, 0x90, 0x12, 0x34, 0x56, 0x78,
///     0x87, 0x65, 0x43, 0x21, 0x50, 0x18, 0x00, 0xFF,
///     0xF0, 0x0D, 0x00, 0x00,
/// ];
/// let tcp = Tcp::try_from(&raw_bytes[..]);
/// println!("{:?}", tcp);
/// ```
impl TryFrom<&[u8]> for Tcp {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 20 {
            panic!(
                "TCP Header must be at least 20 bytes, received: {}",
                bytes.len()
            );
        }

        Ok(Self {
            source_port: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
            dest_port: u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
            seq_num: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            ack_num: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            flags: TcpFlags::from_bits(bytes[13]).unwrap(),
            window_size: u16::from_be_bytes(bytes[14..16].try_into().unwrap()),
            checksum: u16::from_be_bytes(bytes[16..18].try_into().unwrap()),
        })
    }
}

pub struct TcpBuilder {
    source_port: u16,
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    flags: TcpFlags,
    window_size: u16,
}

impl TcpBuilder {
    pub fn new() -> Self {
        Self {
            source_port: 0,
            dest_port: 0,
            seq_num: 0,
            ack_num: 0,
            flags: TcpFlags::UNINT,
            window_size: 1024, // Default
        }
    }

    pub fn source_port(&mut self, port: u16) -> &mut Self {
        self.source_port = port;
        self
    }

    pub fn dest_port(&mut self, port: u16) -> &mut Self {
        self.dest_port = port;
        self
    }

    pub fn seq_num(&mut self, seq: u32) -> &mut Self {
        self.seq_num = seq;
        self
    }

    pub fn ack_num(&mut self, ack: u32) -> &mut Self {
        self.ack_num = ack;
        self
    }

    pub fn flags(&mut self, flags: TcpFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    pub fn window_size(&mut self, size: u16) -> &mut Self {
        self.window_size = size;
        self
    }

    pub fn build(&self, src_ip: Ipv4Addr, dst_ip: Ipv4Addr, payload: &[u8]) -> Tcp {
        let mut tcp = Tcp {
            source_port: self.source_port,
            dest_port: self.dest_port,
            seq_num: self.seq_num,
            ack_num: self.ack_num,
            flags: self.flags,
            checksum: 0,
            window_size: self.window_size,
        };

        // Calculate checksum for the whole tcp packet.
        let checksum = tcp.calculate_checksum(src_ip, dst_ip, payload);
        tcp.checksum = checksum;

        tcp
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_tcp() -> Tcp {
        Tcp {
            source_port: 49320,
            dest_port: 8080,
            seq_num: 305419896,
            ack_num: 2271560481,
            flags: TcpFlags::SYN | TcpFlags::ACK,
            window_size: 255,
            checksum: 61453,
        }
    }

    #[test]
    fn test_tcp_headers_from_bytes() {
        let raw_bytes: [u8; 20] = [
            0xC0, 0xA8, // Source Port: 49320 (0xC0A8 in hex)
            0x1F, 0x90, // Destination Port: 8080 (0x1F90 in hex)
            0x12, 0x34, 0x56, 0x78, // Sequence Number: 305419896 (0x12345678)
            0x87, 0x65, 0x43, 0x21, // Acknowledgment Number: 2271560481 (0x87654321)
            0x50, // Data Offset (4 bits): 5 (20 bytes), Reserved (3 bits): 0, Flags (9 bits): 0b00000000
            0x02, // Flags: SYN (0b00011000)
            0x00, 0xFF, // Window Size: 255
            0xF0, 0x0D, // Checksum: 61453 (0xF00D in hex)
            0x00, 0x00, // Urgent Pointer: 0
        ];

        let headers = Tcp::try_from(&raw_bytes[..]).unwrap();

        assert_eq!(headers.source_port, 49320);
        assert_eq!(headers.dest_port, 8080);
        assert_eq!(headers.seq_num, 305419896);
        assert_eq!(headers.ack_num, 2271560481);
        assert!(headers.flags.contains(TcpFlags::SYN));
        assert_eq!(headers.window_size, 255);
        assert_eq!(headers.checksum, 61453);
    }

    #[test]
    fn test_tcp_headers_to_bytes() {
        let raw_bytes = get_tcp().to_bytes();

        assert_eq!(
            raw_bytes,
            [
                0xC0, 0xA8, // Source Port: 49320 (0xC0A8 in hex)
                0x1F, 0x90, // Destination Port: 8080 (0x1F90 in hex)
                0x12, 0x34, 0x56, 0x78, // Sequence Number: 305419896 (0x12345678)
                0x87, 0x65, 0x43, 0x21, // Acknowledgment Number: 2271560481 (0x87654321)
                0x50, // Data Offset (4 bits): 5 (20 bytes), Reserved (3 bits): 0, Flags (9 bits): 0b00000000
                0x12, // Flags: SYN (0b00011000)
                0x00, 0xFF, // Window Size: 255
                0xF0, 0x0D, // Checksum: 61453 (0xF00D in hex)
                0x00, 0x00, // Urgent Pointer: 0
            ]
        )
    }

    #[test]
    fn test_headers_build_packet_payload() {
        let payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let packet = get_tcp().build_packet(payload);

        assert_eq!(&packet[20..], payload); // Ensure payload is added
        assert_eq!(packet.len(), 20 + payload.len()); // Total packet size
    }

    #[test]
    fn test_tcp_checksum_calculation() {
        let src_ip = Ipv4Addr::new(192, 168, 1, 1);
        let dst_ip = Ipv4Addr::new(192, 168, 1, 2);
        let payload = b"Hello, TCP!";

        let checksum = get_tcp().calculate_checksum(src_ip, dst_ip, payload);
        assert_ne!(checksum, 0); // Ensure checksum is non-zero
    }
}
