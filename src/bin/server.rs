use std::{collections::HashMap, net::UdpSocket};
fn main() {
    let udp = UdpSocket::bind("0.0.0.0:3000").unwrap();
	let mut map = HashMap::new();
    let mut buf = [0u8; 1500];
    while let Ok((_size, from)) = udp.recv_from(&mut buf) {
		let from_id = u32::from_be_bytes(unsafe{
			let mut bytes = [0u8;4];
			let slice = &buf[..4];
			std::ptr::copy_nonoverlapping(slice.as_ptr(), bytes.as_mut_ptr(), 4);
			bytes
		});
		let peer_id = u32::from_be_bytes(unsafe{
			let mut bytes = [0u8;4];
			let slice = &buf[4..8];
			std::ptr::copy_nonoverlapping(slice.as_ptr(), bytes.as_mut_ptr(), 4);
			bytes
		});
		map.insert(from_id, from);
		if let Some(v) = map.get(&peer_id){
			println!("pair {v} with {from}");
			udp.send_to(v.to_string().as_bytes(), from)
                .unwrap();
			udp.send_to(from.to_string().as_bytes(), v)
                .unwrap();
			map.remove(&peer_id);
			map.remove(&from_id);
			println!("\r\n");
		}
        // if first.is_none() {
        //     println!("first is {from}");
        //     first = Some(from);
        //     continue;
        // }
        // if let Some(first_addr) = first {
        //     println!("second is {from}");
        //     println!("prepare to send {first_addr} with {from}");
        //     udp.send_to(first_addr.to_string().as_bytes(), from)
        //         .unwrap();
        //     udp.send_to(from.to_string().as_bytes(), first_addr)
        //         .unwrap();
        //     first = None;
        //     println!("\r\n");
        // }
    }
}
