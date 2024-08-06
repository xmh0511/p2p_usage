use std::{collections::HashMap, net::UdpSocket};
fn main() {
    let udp = UdpSocket::bind("0.0.0.0:3000").unwrap();
    let mut map = HashMap::new();
    let mut buf = [0u8; 1500];
    while let Ok((size, from)) = udp.recv_from(&mut buf) {
        if size > 0 && buf[0] == 0 {
            let from_id = u32::from_be_bytes(unsafe {
                let mut bytes = [0u8; 4];
                let slice = &buf[1..5];
                std::ptr::copy_nonoverlapping(slice.as_ptr(), bytes.as_mut_ptr(), 4);
                bytes
            });
            let peer_id = u32::from_be_bytes(unsafe {
                let mut bytes = [0u8; 4];
                let slice = &buf[5..9];
                std::ptr::copy_nonoverlapping(slice.as_ptr(), bytes.as_mut_ptr(), 4);
                bytes
            });
            map.insert(from_id, from);
            if let Some(v) = map.get(&peer_id) {
                println!("pair {v} with {from}");
                let mut for_from = vec![254u8];
                for_from.extend_from_slice(peer_id.to_be_bytes().as_slice());
                for_from.extend_from_slice(v.to_string().as_bytes());
                udp.send_to(&for_from, from).unwrap();

                let mut for_peer = vec![254u8];
                for_peer.extend_from_slice(from_id.to_be_bytes().as_slice());
                for_peer.extend_from_slice(from.to_string().as_bytes());
                udp.send_to(&for_peer, v).unwrap();
                map.remove(&peer_id);
                map.remove(&from_id);
                println!("\r\n");
            }
        } else if size > 0 && buf[0] == 255 {
            if let Err(_) = udp.send_to(&[255u8], from) {
                if let Some(v) = map.iter().find(|i| i.1 == &from) {
                    let key = *v.0;
                    map.remove(&key);
                }
            }
        }
    }
}
