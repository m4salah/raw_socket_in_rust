use nix::sys::socket::{
    sendto, socket, AddressFamily, SockFlag, SockProtocol, SockType, SockaddrIn,
};
use std::{net::Ipv4Addr, os::fd::AsRawFd};

#[repr(C)]
struct IpHeader {
    version_ihl: u8,
    tos: u8,
    total_len: u16,
    id: u16,
    flags_fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    src_addr: u32,
    dst_addr: u32,
}

#[repr(C)]
struct IcmpHeader {
    icmp_type: u8,
    code: u8,
    checksum: u16,
    id: u16,
    seq_num: u16,
    content: [u8; 2],
}

fn main() -> nix::Result<()> {
    // Create a raw socket
    let sock = socket(
        AddressFamily::Inet,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::Raw,
    )?;

    // Prepare the destination address
    let dest_addr = SockaddrIn::from(std::net::SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0));

    let packet = create_packet();

    // Send the packet
    sendto(
        sock.as_raw_fd(),
        &packet,
        &dest_addr,
        nix::sys::socket::MsgFlags::empty(),
    )?;

    println!("ICMP packet sent successfully");

    Ok(())
}

fn create_packet() -> Vec<u8> {
    let ip_header = IpHeader {
        version_ihl: (4 << 4) | 5,
        tos: 0,
        total_len: (20 + 10), // 20 bytes for IP, 10 for ICMP
        id: 0,
        flags_fragment_offset: 0,
        ttl: 64,
        protocol: 1, // ICMP
        checksum: 0,
        src_addr: 0, // Will be filled by the kernel
        dst_addr: u32::from(Ipv4Addr::new(127, 0, 0, 1)),
    };

    let mut icmp = IcmpHeader {
        icmp_type: 8,
        code: 0,
        checksum: 0,
        id: 0,
        seq_num: 0,
        content: [0xC0, 0xDE],
    };

    // Calculate ICMP checksum
    let icmp_bytes = unsafe {
        &std::slice::from_raw_parts(
            &icmp as *const IcmpHeader as *const u8,
            std::mem::size_of::<IcmpHeader>(),
        )
        .to_vec()
    };
    icmp.checksum = checksum(&icmp_bytes);

    // Combine IP header and ICMP
    let mut packet = unsafe {
        std::slice::from_raw_parts(
            &ip_header as *const IpHeader as *const u8,
            std::mem::size_of::<IpHeader>(),
        )
        .to_vec()
    };
    packet.extend_from_slice(&icmp_bytes);

    packet
}

fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    while i < data.len() - 1 {
        sum += u32::from(data[i + 1]) << 8 | u32::from(data[i]);
        i += 2;
    }
    if i < data.len() {
        sum += u32::from(data[i]) << 8;
    }
    sum = (sum >> 16) + (sum & 0xffff);
    sum += sum >> 16;
    !sum as u16
}
