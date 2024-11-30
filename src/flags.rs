use bitflags::bitflags;
use std::fmt;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct TcpFlags: u8 {
        const UNINT = 0x00;
        const FIN = 0x01;
        const SYN = 0x02;
        const RST = 0x04;
        const PSH = 0x08;
        const ACK = 0x10;
        const URG = 0x20;
        const ECE = 0x40;
        const CWR = 0x80;
    }
}

impl fmt::Display for TcpFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = vec![];

        if self.contains(TcpFlags::FIN) {
            flags.push("FIN");
        }
        if self.contains(TcpFlags::SYN) {
            flags.push("SYN");
        }
        if self.contains(TcpFlags::RST) {
            flags.push("RST");
        }
        if self.contains(TcpFlags::PSH) {
            flags.push("PSH");
        }
        if self.contains(TcpFlags::ACK) {
            flags.push("ACK");
        }
        if self.contains(TcpFlags::URG) {
            flags.push("URG");
        }
        if self.contains(TcpFlags::ECE) {
            flags.push("ECE");
        }
        if self.contains(TcpFlags::CWR) {
            flags.push("CWR");
        }

        if flags.is_empty() {
            write!(f, "UNINT {}", self.bits())
        } else {
            write!(f, "{} {}", flags.join(" | "), self.bits())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_flags() {
        let flags = TcpFlags::SYN | TcpFlags::ACK;
        assert!(flags.contains(TcpFlags::SYN));
        assert!(flags.contains(TcpFlags::ACK));
    }

    #[test]
    fn test_flags_insert() {
        let mut flags = TcpFlags::SYN | TcpFlags::ACK;
        flags.insert(TcpFlags::FIN);
        assert!(flags.contains(TcpFlags::FIN));
    }

    #[test]
    fn teset_flags_remove() {
        let mut flags = TcpFlags::SYN | TcpFlags::FIN;
        flags.remove(TcpFlags::FIN);
        assert!(!flags.contains(TcpFlags::FIN));
    }
}
