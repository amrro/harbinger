use core::panic;
use std::{
    io::{self},
    mem::MaybeUninit,
    net::Ipv4Addr,
};

use socket2::{Domain, SockAddr, Socket, Type};

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
    println!(
        "Recieved {} bytes from {:?}: {:?}",
        bytes_read, sender_addr, recieved_data
    );

    Ok(())
}
