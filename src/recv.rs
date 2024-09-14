use nix::sys::socket::{socket, AddressFamily, SockFlag, SockProtocol, SockType};
use nix::unistd::read;
use std::os::unix::io::AsRawFd;

fn main() {
    // Create a raw socket for ICMP
    let sock = socket(
        AddressFamily::Inet,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::Icmp,
    )
    .expect("Failed to create raw socket for ICMP"); // TODO: Handle error();

    println!("Listening for ICMP packets...");

    loop {
        let mut buf = [0u8; 1024];
        match read(sock.as_raw_fd(), &mut buf) {
            Ok(num_read) => {
                println!("Received packet:");
                for byte in &buf[..num_read] {
                    print!("{:02X} ", byte);
                }
                println!();
            }
            Err(err) => {
                eprintln!("Error reading from socket: {}", err);
                break;
            }
        }
    }
}
