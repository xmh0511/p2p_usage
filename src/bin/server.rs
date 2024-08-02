use std::net::UdpSocket;
fn main() {
    let udp = UdpSocket::bind("0.0.0.0:3000").unwrap();
    let mut buf = [0u8; 1500];
    let mut first = None;
    while let Ok((_size, from)) = udp.recv_from(&mut buf) {
        if first.is_none() {
            first = Some(from);
            continue;
        }
        if let Some(first_addr) = first {
            println!("prepare to send");
            udp.send_to(first_addr.to_string().as_bytes(), from)
                .unwrap();
            udp.send_to(from.to_string().as_bytes(), first_addr)
                .unwrap();
            first = None;
        }
    }
}
