use core::fmt;

pub struct TcpFlag(pub u8);

// A bitmasking flags.
impl TcpFlag {
    const CWR: u8 = 0x80;
    const ECE: u8 = 0x40;
    const URG: u8 = 0x20;
    const ACK: u8 = 0x10;
    const PSH: u8 = 0x08;
    const RST: u8 = 0x04;
    const SYN: u8 = 0x02;
    const FIN: u8 = 0x01;

    pub fn from_byte(byte: u8) -> Self {
        Self(byte)
    }

    /// Return a list of active flags in a form of strings for representations.
    fn names(&self) -> Vec<&'static str> {
        let mut names = vec![];

        if self.has_flag(Self::CWR) {
            names.push("CWR");
        }
        if self.has_flag(Self::ECE) {
            names.push("ECE");
        }
        if self.has_flag(Self::URG) {
            names.push("URG");
        }
        if self.has_flag(Self::ACK) {
            names.push("ACK");
        }
        if self.has_flag(Self::PSH) {
            names.push("PSH");
        }
        if self.has_flag(Self::RST) {
            names.push("RST");
        }
        if self.has_flag(Self::SYN) {
            names.push("SYN");
        }
        if self.has_flag(Self::FIN) {
            names.push("FIN");
        }
        names
    }

    fn has_flag(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

impl fmt::Display for TcpFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self.names();
        write!(f, "{:x} {}", self.0, names.join(", "))
    }
}
