use ctrlc2;
use std::net::{IpAddr, SocketAddrV4};
use std::sync::mpsc;

use p2p_channel::{channel::Route, punch::NatType};
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    _ = args.get(1).expect("填写身份参数");
    let (mut channel, mut punch, idle) =
        p2p_channel::boot::Boot::new::<String>(20, 9000, 0).unwrap();
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
    let (tx, sig_rx) = mpsc::channel();
    let handle = ctrlc2::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
        true
    })
    .expect("Error setting Ctrl-C handler");

    let t1 = {
        // 空闲处理，添加的路由空闲时触发
        std::thread::spawn(move || {
            loop {
                let (idle_status, id, _route) = match idle.next_idle() {
                    Ok(r) => r,
                    Err(_) => {
                        println!("idle task exit {}", line!());
                        break;
                    }
                };
                // channel.send_to_route()
                //channel.remove_route(&id[0]);
                println!("idle {:?} {}", idle_status, &id[0]);
            }
        })
    };
    let (t2, t3) = {
        //let ident = args[1].clone();
        //let ident2 = ident.clone();
        // 打洞处理
        let mut punch2 = punch.try_clone().unwrap();
        (
            std::thread::spawn(move || {
                let buf = b"\0";
                loop {
                    let (id, nat_info) = match punch.next_cone(None) {
                        Ok(r) => r,
                        Err(_) => {
                            println!("cone punch task exit {}", line!());
                            break;
                        }
                    };
                    println!("{id} -> {nat_info:?}");
                    match punch.punch(&buf[..], id, nat_info) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("cone punch task exit {}", line!());
                            break;
                        }
                    }
                }
            }),
            std::thread::spawn(move || {
                let buf = b"\0";
                loop {
                    let (id, nat_info) = match punch2.next_symmetric(None) {
                        Ok(r) => r,
                        Err(_) => {
                            println!("symmetric punch task exit {}", line!());
                            break;
                        }
                    };
                    println!("{id} -> {nat_info:?}");
                    match punch2.punch(&buf[..], id, nat_info) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("symmetric punch task exit {}", line!());
                            break;
                        }
                    }
                }
            }),
        )
    };

    let mut buf = [0; 1500];

    let (tx, rx) = std::sync::mpsc::channel::<(IpAddr, u16)>();
	let (tx2,rx2) = std::sync::mpsc::channel::<()>();
	let chanel2 = channel.try_clone().unwrap();
    let chanel3 = channel.try_clone().unwrap();
	let mut chanel1 = channel.try_clone().unwrap();
    std::thread::spawn(move || {
        chanel1
            .send_to_addr(b"c", "150.158.95.11:3000".parse().unwrap())
            .unwrap();
        let (len, _route_key) = chanel1
            .recv_from(&mut buf, None)
            .unwrap();
        let addr = String::from_utf8_lossy(&buf[..len]).to_string();
        //println!("{addr}");
        let s: SocketAddrV4 = addr.parse().unwrap();
        println!("peer addr: {}", s);
        let public_ip = std::net::IpAddr::V4(s.ip().to_owned());
        let public_port = s.port();
        tx.send((public_ip, public_port)).unwrap();
		tx2.send(()).unwrap();
    });
    // Do something...
    let t4 = std::thread::spawn(move || {
        let (public_ip, public_port) = rx.recv().unwrap();
        let id2 = format!("{public_ip}:{public_port}");
        loop {
            if let Err(_) = chanel2.punch(
                id2.clone(),
                p2p_channel::punch::NatInfo::new(
                    vec![public_ip],
                    public_port,
                    0,
                    public_ip,
                    public_port,
                    NatType::Cone, //对方的Nat网络类型
                ),
            ) {
                println!("trigger punch task exit {}", line!());
                break;
            }; //触发打洞
            if let Err(_) = chanel2.punch(
                id2.clone(),
                p2p_channel::punch::NatInfo::new(
                    vec![public_ip],
                    public_port,
                    0,
                    public_ip,
                    public_port,
                    NatType::Symmetric, //对方的Nat网络类型
                ),
            ) {
                println!("trigger punch task exit {}", line!());
                break;
            }; //触发打洞
            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    });

    let t5 = std::thread::spawn(move || {
        let mut status = false;
		rx2.recv().unwrap();
        // 接收数据处理
        loop {
            let (len, route_key) = match channel.recv_from(&mut buf, None) {
                Ok(r) => r,
                Err(_) => {
                    println!("receive data task exit {}", line!());
                    break;
                }
            };
			if len>0 && buf[0] == 0{
				println!("say hello");
			}else if len >0 && buf[0]==1{
				let text = String::from_utf8_lossy(&buf[1..len]).to_string();
				println!("receive {text} {route_key:?}");
			}
            let id = format!("{}", route_key.addr);
            if channel.route_to_id(&route_key).is_none() {
                channel.add_route(id.clone(), Route::from(route_key, 10, 64)); //超时触发空闲
            }
            if !status {
                let msg = format!("my name is {}", args[1]);
				let mut msg_p = vec![1u8];
				msg_p.extend_from_slice(msg.as_bytes());
                //channel.send_to_route(msg.as_bytes(), &route_key).unwrap();
                //println!("route table {:?}", channel.route_table());
                _ = channel.send_to_id(msg_p.as_slice(), &id).unwrap();
                //channel.send_to_addr(msg.as_bytes(), addr.parse().unwrap()).unwrap();
                status = true;
            }
        }
    });
    sig_rx.recv().expect("Could not receive from channel.");
    println!("exit");
    chanel3.close().unwrap();
    handle.join().unwrap();
}
