use std::{os::fd::AsRawFd, str::FromStr};

use nix::{
    sys::socket::{
        bind, recvfrom, socket, AddressFamily, SockFlag, SockProtocol, SockType, SockaddrIn,
    },
    unistd::close,
};

// https://en.wikipedia.org/wiki/IPv4#/media/File:IPv4_Packet-en.svg
#[repr(C)]
struct Ipv4Header {
    // first 4 bits is the version of the protocol
    // second 4 bits is the IHL
    version_ihl: u8,
    tos: u8,
    total_length: u16,
    id: u16,
    flags_fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    header_checksum: u16,
    src_addr: [u8; 4],
    dst_addr: [u8; 4],
}

fn main() {
    let socket_fd = socket(
        AddressFamily::Inet,
        SockType::Raw,
        SockFlag::empty(),
        Some(SockProtocol::Icmp),
    )
    .expect("Failed to open socket file descriptor");

    let addr = SockaddrIn::from_str("127.0.0.1:8080").expect("Failed to make the address");

    bind(socket_fd.as_raw_fd(), &addr).expect("Failed to bind socket");
    println!("Listening for ICMP packets...");
    let mut buf = [0u8; 1024];
    loop {
        match recvfrom::<SockaddrIn>(socket_fd.as_raw_fd(), &mut buf) {
            Ok((size, _)) => {
                if size >= std::mem::size_of::<Ipv4Header>() {
                    let header: Ipv4Header = unsafe { std::ptr::read(buf.as_ptr() as *const _) };
                    println!("Received packet:{}", size);
                    println!(
                        "  Source: {}.{}.{}.{}",
                        header.src_addr[0],
                        header.src_addr[1],
                        header.src_addr[2],
                        header.src_addr[3]
                    );
                    println!(
                        "  Destination: {}.{}.{}.{}",
                        header.dst_addr[0],
                        header.dst_addr[1],
                        header.dst_addr[2],
                        header.dst_addr[3]
                    );
                    println!("  Protocol: {}", header.protocol);
                    println!("  TTL: {}", header.ttl);

                    // Print ICMP payload
                    if size > std::mem::size_of::<Ipv4Header>() {
                        println!(
                            "  ICMP Payload: {:?}",
                            &buf[std::mem::size_of::<Ipv4Header>()..size]
                        );
                    }
                }
            }
            Err(err) => {
                eprintln!("Error receiving packet: {}", err);
                break;
            }
        }
    }

    close(socket_fd.as_raw_fd()).unwrap();
}
