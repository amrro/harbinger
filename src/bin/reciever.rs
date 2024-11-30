use core::panic;
use harbinger::tcp::Tcp;
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    io::{self},
    mem::MaybeUninit,
    net::Ipv4Addr,
};

fn main() -> io::Result<()> {
    let receiver = Socket::new(Domain::IPV4, Type::RAW, None)
        .unwrap_or_else(|e| panic!("Failed to create a recevier socket.\n{}", e));

    let local_ip = Ipv4Addr::new(127, 0, 0, 1);
    let receiver_sock_addr = SockAddr::from(std::net::SocketAddr::new(local_ip.into(), 0));
    receiver
        .bind(&receiver_sock_addr)
        .unwrap_or_else(|e| panic!("Failed to bind to receiver raw socket: {}", e));

    // Recieving a packet.
    let mut buffer: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
    let (bytes_read, sender_addr) = receiver
        .recv_from(&mut buffer)
        .unwrap_or_else(|e| panic!("Failed to recv_from: {}", e));

    let recieved_data =
        unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, bytes_read) };

    let ip_header_len = ((recieved_data[0] & 0x0F) * 4) as usize;
    let tcp_data = &recieved_data[ip_header_len..];

    let (tcp, payload) = Tcp::parse_packet(tcp_data);
    println!(
        "Recieved {} bytes from {:?}: {}",
        bytes_read, sender_addr, tcp
    );

    if let Some(pay) = payload {
        println!("\n{}", pay);
    }

    Ok(())
}
