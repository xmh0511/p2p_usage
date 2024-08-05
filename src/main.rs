use std::net::SocketAddrV4;

use p2p_channel::{channel::Route, punch::NatType};
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    _ = args.get(1).expect("填写身份参数");
    let (mut channel, mut punch, idle) =
        p2p_channel::boot::Boot::new::<String>(110, 9000, 0).unwrap();
    //channel.set_nat_type(NatType::Cone).unwrap();  //一定要根据本地环境的Nat网络类型设置
    let nat = channel
        .set_nat_type_with_stun(
            vec![
                "stun.miwifi.com:3478".to_string(),
                "stun.chat.bilibili.com:3478".to_string(),
                "stun.hitv.com:3478".to_string(),
                "stun.cdnbye.com:3478".to_string(),
                "stun.tel.lu:3478".to_string(),
                "stun.smartvoip.com:3478".to_string(),
            ],
            None,
        )
        .unwrap();
    println!("nat info: {:?}", nat);
    {
        // 空闲处理，添加的路由空闲时触发
        std::thread::spawn(move || {
            loop {
                let (idle_status, id, _route) = idle.next_idle().unwrap();
                // channel.send_to_route()
                //channel.remove_route(&id[0]);
                println!("idle {:?} {}", idle_status, &id[0]);
            }
        });
    }
    {
        let ident = args[1].clone();
        let ident2 = ident.clone();
        // 打洞处理
        let mut punch2 = punch.try_clone().unwrap();
        std::thread::spawn(move || {
            let buf = format!("hello from {}", ident);
            loop {
                let (id, nat_info) = punch.next_cone(None).unwrap();
                println!("{id} -> {nat_info:?}");
                punch.punch(buf.as_bytes(), id, nat_info).unwrap();
            }
        });
        std::thread::spawn(move || {
            let buf = format!("hello from {}", ident2);
            loop {
                let (id, nat_info) = punch2.next_symmetric(None).unwrap();
                println!("{id} -> {nat_info:?}");
                punch2.punch(buf.as_bytes(), id, nat_info).unwrap();
            }
        });
    }

    let mut buf = [0; 1500];

    channel
        .send_to_addr(b"c", "150.158.95.11:3000".parse().unwrap())
        .unwrap();
    let (len, _route_key) = channel.recv_from(&mut buf, None).unwrap();
    let addr = String::from_utf8_lossy(&buf[..len]).to_string();
    //println!("{addr}");
    let s: SocketAddrV4 = addr.parse().unwrap();
    println!("peer addr: {}", s);
    let public_ip = std::net::IpAddr::V4(s.ip().to_owned());
    let public_port = s.port();
    let id = format!("{s}");
    // Do something...
    let chanel2 = channel.try_clone().unwrap();
    let id2 = id.clone();
    std::thread::spawn(move || {
        loop {
            chanel2
                .punch(
                    id2.clone(),
                    p2p_channel::punch::NatInfo::new(
                        vec![public_ip],
                        public_port,
                        0,
                        public_ip,
                        public_port,
                        NatType::Cone, //对方的Nat网络类型
                    ),
                )
                .unwrap(); //触发打洞
            chanel2
                .punch(
                    id2.clone(),
                    p2p_channel::punch::NatInfo::new(
                        vec![public_ip],
                        public_port,
                        0,
                        public_ip,
                        public_port,
                        NatType::Symmetric, //对方的Nat网络类型
                    ),
                )
                .unwrap(); //触发打洞
            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    });

    let mut status = false;
    // 接收数据处理
    loop {
        let (len, route_key) = channel.recv_from(&mut buf, None).unwrap();
        let text = String::from_utf8_lossy(&buf[..len]).to_string();
        println!("receive {text} {route_key:?}");
        channel.add_route(id.clone(), Route::from(route_key, 10, 64)); //超时触发空闲
        if !status {
            let msg = format!("my name is {}", args[1]);
            //channel.send_to_route(msg.as_bytes(), &route_key).unwrap();
            //println!("route table {:?}", channel.route_table());
            _ = channel.send_to_id(msg.as_bytes(), &id).unwrap();
            //channel.send_to_addr(msg.as_bytes(), addr.parse().unwrap()).unwrap();
            status = true;
        }
    }
}
