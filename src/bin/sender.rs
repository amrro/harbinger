use std::net::Ipv4Addr;

use harbinger::tcp_headers::TcpHeader;
use socket2::{Domain, SockAddr, Socket, Type};

fn main() {
    // Create a raw socket of IPV4 (no specific protocl, using custom!).
    // TODO: (later) try `Protocl::TCP`.
    let sender =
        Socket::new(Domain::IPV4, Type::RAW, None).expect("Failed to create sender socket");

    // Set the target IP address.
    let target_ip = Ipv4Addr::new(127, 0, 0, 1);
    let target_sock_addr = SockAddr::from(std::net::SocketAddr::new(target_ip.into(), 0));

    let tcp = TcpHeader {
        source_port: 0,
        dest_port: 0,
        seq_num: 1,
        ack_num: 0,
        flags: 0x02, // SYN
        window_size: 1024,
        checksum: 0, // Placeholder
    };

    // Construct a raw payload (custom protocol, 0xABCD, for example)
    let payload = b"hello, raw TCP!";
    let packet = tcp.build_packet(payload);

    sender
        .send_to(&packet, &target_sock_addr)
        .unwrap_or_else(|e| panic!("Failed to send to addr: {:?},\n{}", target_sock_addr, e));
}
