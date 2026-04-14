
use std::io::{
    self,
};
use std::net::{
    SocketAddr,
    UdpSocket,
};
use std::time::{
    Duration,
    Instant,
};
use std::sync::mpsc::{
    channel,
};

type Map<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug)]
pub struct ServerCfg {
    pub listen_addr: SocketAddr,
    pub target_addr: SocketAddr,
    pub nonblocking: bool,
}

pub fn server(cfg: ServerCfg) -> ! {
    // println!("cfg = {:#?}", cfg);

    let ServerCfg {
        listen_addr,
        target_addr,
        nonblocking,
    } = cfg;

    let port1 = UdpSocket::bind(listen_addr).unwrap();
    let port2 = UdpSocket::bind((listen_addr.ip(), listen_addr.port()+1)).unwrap();

    println!("Server proxy for {}", target_addr);
    println!("Listening on {}:{{{}, {}}}", listen_addr.ip(), listen_addr.port(), listen_addr.port()+1);

    std::thread::scope(|s| {
        s.spawn(|| listen(port1, target_addr, nonblocking));
        s.spawn(|| listen(port2, (target_addr.ip(), target_addr.port()+1).into(), nonblocking));
    });

    println!("ERROR: Something went wrong. Restart is required.");

    loop {
        std::thread::sleep(Duration::MAX);
    }
}

const BUF_SIZE: usize = 64*1024;
const TIMEOUT_SECS: Duration = Duration::from_secs(10);
const CLEANUP_PERIOD_SECS: Duration = Duration::from_secs(TIMEOUT_SECS.as_secs() / 2);

fn listen(client: UdpSocket, server_addr: SocketAddr, nonblocking: bool) {
    let mut buf = vec![0u8; BUF_SIZE];
    let mut peers: Map<SocketAddr, UdpSocket> = Map::new();
    let mut last_cleanup = Instant::now();
    let (tx, rx) = channel();
    let local_port = client.local_addr().unwrap().port();
    /*
        TODO: spin_loop()?
    */
    client.set_nonblocking(nonblocking).unwrap();
    /*
        TODO: Platforms may return a different error code whenever a read times out as a
        result of setting this option. For example Unix typically returns an error of the
        kind WouldBlock, but Windows may return TimedOut.
    */
    client.set_read_timeout(Some(CLEANUP_PERIOD_SECS)).unwrap();
    loop {
        match client.recv_from(&mut buf) {
            Ok((n, peer)) => {
                let data = &buf[..n];
                let send_data = |socket: &UdpSocket| {
                    if let Err(e) = socket.send(data) {
                        println!("ERROR [{}]: proxy-server send: {}", peer, e);
                        return false;
                    }
                    true
                };

                if let Some(server) = peers.get(&peer) {
                    send_data(server);
                    continue;
                }

                println!("INCOMING {} -> {}", peer, local_port);

                let server = match UdpSocket::bind(SocketAddr::from(([0; 4], 0))) {
                    Ok(__) => __,
                    Err(e) => {
                        println!("ERROR [{}]: could not open a proxy-server socket: {}", peer, e);
                        continue;
                    }
                };

                server.connect(server_addr).unwrap();
                server.set_nonblocking(nonblocking).unwrap();
                server.set_read_timeout(Some(TIMEOUT_SECS)).unwrap();

                /*
                    TODO: Retry on failure?
                */
                if !send_data(&server) {
                    continue;
                }

                peers.insert(peer, server.try_clone().unwrap());

                let client = client.try_clone().unwrap();
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut buf = vec![0u8; BUF_SIZE];
                    loop {
                        match server.recv(&mut buf) {
                            Ok(n) => {
                                let data = &buf[..n];
                                if let Err(e) = client.send_to(data, peer) {
                                    println!("ERROR [{}]: proxy-client send: {}", peer, e);
                                }
                            }
                            Err(e) => {
                                if e.kind() != io::ErrorKind::WouldBlock {
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
                    }
                });
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    if e.kind() != io::ErrorKind::TimedOut && e.kind() != io::ErrorKind::ConnectionReset {
                        println!("ERROR: client-proxy recv: {}", e);
                    }
                }
            }
        }

        let time = Instant::now();

        if time.duration_since(last_cleanup) >= CLEANUP_PERIOD_SECS {
            for peer in rx.try_iter() {
                println!("TIMED OUT {}", peer);
                peers.remove(&peer);
            }
            last_cleanup = time;
        }
    }
}
