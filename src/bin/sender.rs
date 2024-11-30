use harbinger::{flags::TcpFlags, tcp::TcpBuilder};
use socket2::{Domain, SockAddr, Socket, Type};
use std::net::Ipv4Addr;

fn main() {
    let sender =
        Socket::new(Domain::IPV4, Type::RAW, None).expect("Failed to create sender socket");
    let src_ip = Ipv4Addr::new(127, 0, 0, 1);
    // Set the target IP address.
    let target_ip = Ipv4Addr::new(127, 0, 0, 1);
    let target_sock_addr = SockAddr::from(std::net::SocketAddr::new(target_ip.into(), 0));

    let payload = b"Hello, TCP!";
    let tcp = TcpBuilder::new()
        .source_port(0)
        .dest_port(0)
        .seq_num(305419896)
        .ack_num(2271560481)
        .flags(TcpFlags::SYN)
        .window_size(255)
        .build(src_ip, target_ip, payload);

    // Construct a raw payload (custom protocol, 0xABCD, for example)
    let payload = b"hello, raw TCP!";
    let packet = tcp.build_packet(payload);

    sender
        .send_to(&packet, &target_sock_addr)
        .unwrap_or_else(|e| panic!("Failed to send to addr: {:?},\n{}", target_sock_addr, e));
}
