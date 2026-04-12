
use std::io::{
    self,
};
use std::net::{
    SocketAddr,
    UdpSocket,
};
use std::time::{
    Duration,
};
use std::sync::mpsc::{
    channel,
};

type Map<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug)]
pub struct ServerCfg {
    pub listen_addr: SocketAddr,
    pub target_addr: SocketAddr,
}

pub fn server(cfg: ServerCfg) -> ! {
    // println!("cfg = {:#?}", cfg);

    let ServerCfg {
        listen_addr,
        target_addr,
    } = cfg;

    let port1 = UdpSocket::bind(listen_addr).unwrap();
    let port2 = UdpSocket::bind((listen_addr.ip(), listen_addr.port()+1)).unwrap();

    println!("Server proxy for {}", target_addr);
    println!("Listening on {}:{{{}, {}}}", listen_addr.ip(), listen_addr.port(), listen_addr.port()+1);

    std::thread::scope(|s| {
        s.spawn(|| listen(port1, target_addr));
        s.spawn(|| listen(port2, (target_addr.ip(), target_addr.port()+1).into()));
    });

    println!("ERROR: Something went wrong. Restart is required.");

    loop {}
}

const BUF_SIZE: usize = 64*1024;
const RECV_TIMEOUT_SECS: Duration = Duration::from_secs(10);

fn listen(client: UdpSocket, server_addr: SocketAddr) {
    let mut buf = vec![0u8; BUF_SIZE];
    let mut peers: Map<SocketAddr, UdpSocket> = Map::new();
    let (tx, rx) = channel();
    client.set_read_timeout(Some(RECV_TIMEOUT_SECS / 2)).unwrap();
    loop {
        match client.recv_from(&mut buf) {
            Ok((n, peer)) => {
                let data = &buf[..n];

                if let Some(server) = peers.get(&peer) {
                    if let Err(e) = server.send(data) {
                        println!("ERROR [{}]: proxy-server send: {}", peer, e);
                    }
                    continue;
                }

                println!("INCOMING {}", peer);

                let server = match UdpSocket::bind(SocketAddr::from(([0; 4], 0))) {
                    Ok(__) => __,
                    Err(e) => {
                        println!("ERROR [{}]: could not open a proxy-server socket: {}", peer, e);
                        continue;
                    }
                };

                server.connect(server_addr).unwrap();

                if let Err(e) = server.send(data) {
                    println!("ERROR [{}]: proxy-server send: {}", peer, e);
                    continue;
                }

                peers.insert(peer, server.try_clone().unwrap());

                let client = client.try_clone().unwrap();
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut buf = vec![0u8; BUF_SIZE];
                    server.set_read_timeout(Some(RECV_TIMEOUT_SECS)).unwrap();
                    loop {
                        match server.recv(&mut buf) {
                            Ok(n) => {
                                let data = &buf[..n];
                                if let Err(e) = client.send_to(data, peer) {
                                    println!("ERROR [{}]: proxy-client send: {}", peer, e);
                                }
                            }
                            Err(e) => {
                                if e.kind() == io::ErrorKind::TimedOut {
                                    if let Err(e) = tx.send(peer) {
                                        println!("ERROR [{}]: could not send a timed out notification: {}", peer, e);
                                    }
                                    break;
                                }
                                println!("ERROR [{}]: server-proxy recv: {}", peer, e);
                            }
                        }
                    }
                });
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::TimedOut && e.kind() != io::ErrorKind::ConnectionReset {
                    println!("ERROR: client-proxy recv: {}", e);
                }
            }
        }
        for peer in rx.try_iter() {
            println!("TIMED OUT {}", peer);
            peers.remove(&peer);
        }
    }
}
