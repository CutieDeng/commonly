use std::{net::{SocketAddrV4, Ipv4Addr, TcpStream, UdpSocket}, io::{stdin, Write, Read}, fs::File, time::Duration};

fn main() {
    let mut connect_config = Box::new(ConnectConfig::Nothing); 
    let mut input = String::new(); 
    let mut cache = Box::new([0u8; 1024]); 
    loop {
        input.clear(); 
        stdin().read_line(&mut input).unwrap(); 
        if input.starts_with("connect ") {
            let mut values = [0u16; 5]; 
            let mut maximum = 0; 
            let mut tag = false; 
            for (i, r) in input.split_ascii_whitespace().skip(1).map(|s| s.parse::<u16>()).enumerate() {
                maximum = i; 
                if i >= 5 {
                    tag = true; 
                    println!("\x1b[33;1m[!] warning: too many arguments for connect operator! \x1b[0m"); 
                    break 
                }
                values[i] = match r {
                    Ok(v) => {v},
                    Err(e) => {
                        tag = true;
                        println!("\x1b[33;1m[!] warning: failing parse at the index {i} - {:?}\x1b[0m", e); 
                        break 
                    },
                }
            }
            if !tag {
                if maximum != 4 {
                    println!("\x1b[33;1m[!] warning: too less arguments for connect operator! \x1b[0m");    
                } else {
                    let mut ipv4 = [0u8; 4]; 
                    let mut tag = false; 
                    for i in 0..ipv4.len() {
                        ipv4[i] = match values[i].try_into() {
                            Ok(i) => {i},
                            Err(_e) => {
                                println!("\x1b[33;1m[!] warning: failing parse at the index {i} - Int Overflow Error \x1b[0m"); 
                                tag = false; 
                                break; 
                            },
                        }
                    }
                    if !tag {
                        let new_stream = TcpStream::connect(SocketAddrV4::new(
                            Ipv4Addr::new(ipv4[0], ipv4[1], ipv4[2], ipv4[3]),
                            values[4])
                        ); 
                        match new_stream {
                            Ok(n) => {
                                n.set_read_timeout(Some(Duration::from_secs(1))).unwrap(); 
                                // connect_config.target = Some(n); 
                                *connect_config = ConnectConfig::Tcp(n); 
                                println!("[-] connect build. "); 
                            },
                            Err(e) => {
                                println!("\x1b[31;1m[e] error: {:?}\x1b[0m", e)
                            },
                        }
                    }
                }
            }
        } else if let Some (input) = input.strip_prefix("connect2 ") {
            let input = input.trim_end();
            let tcp_stream = TcpStream::connect(input); 
            match tcp_stream {
                Ok(t) => {
                    *connect_config = ConnectConfig::Tcp(t); 
                },
                Err(e) => {
                    println!("\x1b[31;1m[e] error {:?}\x1b[0m", e); 
                },
            }
        } else if let Some(input) = input.strip_prefix("uconnect ") { 
            let others = input.trim_end();
            let u = UdpSocket::bind("0.0.0.0:0").unwrap();
            let u2 = u.connect(others);
            match u2 {
                Ok(()) => {
                    u.set_read_timeout(Some(Duration::from_secs(1))).unwrap(); 
                    *connect_config = ConnectConfig::Udp(u); 
                },
                Err(e) => {
                    println!("\x1b[31;1m[e] error: {:?}\x1b[0m", e); 
                },
            }
        } else if let Some(input) = input.strip_prefix("file ") { 
            match connect_config.as_mut() {
                ConnectConfig::Tcp(ref mut tcp_stream) => { 
                    let input = input.trim_end(); 
                    let f = File::open(input);
                    match f {
                        Ok(mut actual_file) => {
                            loop {
                                let r = actual_file.read(cache.as_mut_slice()).unwrap(); 
                                if r == 0 { 
                                    println!("[-] finish the file block write. "); 
                                    break 
                                }
                                match tcp_stream.write(&cache[0..r]) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        println!("\x1b[31;1m[e] error: write {:?}\x1b[0m", e); 
                                        break 
                                    },
                                }
                            }
                            tcp_read_simple(tcp_stream, cache.as_mut_slice()); 
                        },
                        Err(e) => {
                            println!("\x1b[31;1m[e] error: file open failure - {:?}\x1b[0m", e); 
                        },
                    }
                }
                ConnectConfig::Udp(ref mut udp_socket) => {
                    let input = input.trim_end();
                    let f = File::open(input);
                    match f {
                        Ok(mut actual_file) => {
                            loop {
                                let r = actual_file.read(cache.as_mut_slice()).unwrap(); 
                                if r == 0 { 
                                    println!("[-] finish the file block write. "); 
                                    break 
                                }
                                match udp_socket.send(&cache[0..r]) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        println!("\x1b[31;1m[e] error: write {:?}\x1b[0m", e); 
                                        break 
                                    },
                                }
                            }
                            udp_read_simple(udp_socket, cache.as_mut_slice()); 
                        },
                        Err(e) => {
                            println!("\x1b[31;1m[e] error: file open failure - {:?}\x1b[0m", e); 
                        },
                    }
                }
                ConnectConfig::Nothing => println!("\x1b[31;1m[e] error: invalid status for sending file, because connection doesn't exist. \x1b[0m"), 
            }
        } else if input.starts_with(' ') {
            // match connect_config.as_mut().target {
            match connect_config.as_mut() { 
                ConnectConfig::Tcp(ref mut tcp_stream) => { 
                // Some(ref mut tcp_stream) => {
                    match tcp_stream.write(input[1..].as_bytes()) {
                        Ok(_) => {
                        },
                        Err(e) => {
                            println!("\x1b[31;1m[e] error: {:?}\x1b[0m", e) 
                        }
                    }
                    tcp_read_simple(tcp_stream, cache.as_mut_slice()); 
                }
                ConnectConfig::Nothing => println!("\x1b[31;1m[e] error: invalid status for sending info, because connection doesn't exist. \x1b[0m"), 
                ConnectConfig::Udp(ref mut udp_socket) => {
                    // udp_socket.send_to(buf, addr)
                    match udp_socket.send(input[1..].as_bytes()) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("\x1b[31;1m[e] error: {:?}\x1b[0m", e)
                        },
                    }
                    udp_read_simple(udp_socket, cache.as_mut_slice()); 
                }
            }
        } else if input.starts_with("help") {
            println!("cutie-nc, version {}\n", env!("CARGO_PKG_VERSION")); 
            println!("{}", include_str!("my-docs/cutie-nc-tutorial.txt")); 
        } else if input.starts_with("exit") {
            println!("exit. "); 
            break 
        } else {
            println!("\x1b[33;1m{}{}\x1b[0m", "[!] warning: unexpected format for the raw str input: ", input.trim_end())
        }
    }
}

// struct ConnectConfig {
//     target: Option<TcpStream>, 
// }

enum ConnectConfig {
    Nothing, 
    Tcp(TcpStream), 
    Udp(UdpSocket), 
} 

fn tcp_read_simple(value: &mut TcpStream, cache: &mut [u8]) {
    let mut continue_flag = true; 
    while continue_flag {
        match value.read(cache.as_mut()) {
            Ok(s) => {
                if s < cache.len() {
                    continue_flag = false; 
                }
                println!("[-] info: read replies from server. "); 
                let c = String::from_utf8_lossy(&cache[..s]); 
                println!("{}", c); 
            },
            Err(e) => {
                continue_flag = false; 
                println!("[-] info: read failure - {:?}", e)
            },
        } 
    }
}

fn udp_read_simple(value: &mut UdpSocket, cache: &mut [u8]) {
    let mut continue_flag = true; 
    while continue_flag {
        match value.recv(cache) {
            Ok(s) => {
                if s < cache.len() { continue_flag = false } 
                println!("[-] info: read replies from server. "); 
                let c = String::from_utf8_lossy(&cache[..s]); 
                println!("{}", c); 
            }
            Err(e) => {
                continue_flag = false; 
                println!("[-] info: read failure - {:?}", e); 
            }
        }
    }
}