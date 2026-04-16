
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

    let port1 = UdpSocket::bind(listen_addr).map_err(|e| {
        println!("ERROR: could not open a proxy<->client port {}: {}", listen_addr.port(), e);
    }).ok();
    let port2 = UdpSocket::bind((listen_addr.ip(), listen_addr.port()+1)).map_err(|e| {
        println!("ERROR: could not open a proxy<->client port {}: {}", listen_addr.port()+1, e);
    }).ok();

    if let (Some(port1), Some(port2)) = (port1, port2) {
        println!("Server proxy for {}", target_addr);
        println!("Listening on {}:{{{}, {}}}", listen_addr.ip(), listen_addr.port(), listen_addr.port()+1);

        std::thread::scope(|s| {
            s.spawn(|| listen::<false>(port1, target_addr, nonblocking));
            s.spawn(|| listen::< true>(port2, (target_addr.ip(), target_addr.port()+1).into(), nonblocking));
        });
    }

    println!("ERROR: Something went wrong. Restart is required.");

    loop {
        std::thread::sleep(Duration::MAX);
    }
}

const BUF_SIZE: usize = 64*1024;
const TIMEOUT_SECS: Duration = Duration::from_secs(10);
const CLEANUP_PERIOD_SECS: Duration = Duration::from_secs(TIMEOUT_SECS.as_secs() / 2);

const SUFFIX: &[u8] = b"\x1B\xFF\xFF\xFF [PROXY]";
const OFFSET_TO_PORT: usize = 10;
const OFFSET_TO_NAME: usize = 18;

fn listen<const QUERY: bool>(client: UdpSocket, server_addr: SocketAddr, nonblocking: bool) {
    let mut buf = vec![0u8; BUF_SIZE];
    let mut clients: Map<SocketAddr, UdpSocket> = Map::new();
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
            Ok((n, client_addr)) => {
                let data = &buf[..n];
                let send_data = |socket: &UdpSocket| {
                    if let Err(e) = socket.send(data) {
                        println!("ERROR [{}]: proxy->server send: {}", client_addr, e);
                        return false;
                    }
                    true
                };

                if let Some(server) = clients.get(&client_addr) {
                    send_data(server);
                    continue;
                }

                println!("INCOMING {} -> {}", client_addr, local_port);

                let server = match UdpSocket::bind(SocketAddr::from(([0; 4], 0))) {
                    Ok(__) => __,
                    Err(e) => {
                        println!("ERROR [{}]: could not open a proxy<->server socket: {}", client_addr, e);
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

                clients.insert(client_addr, server.try_clone().unwrap());

                let client = client.try_clone().unwrap();
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut buf = vec![0u8; BUF_SIZE];
                    loop {
                        match server.recv(&mut buf) {
                            Ok(mut n) => {
                                if QUERY && matches!(&buf[..n], [0x80, 0, 0, 0, 0, ..]) {
                                    buf_correct_port(&mut buf, local_port-1);
                                    buf_insert_name_suffix(&mut buf, &mut n);
                                }
                                let data = &buf[..n];
                                if let Err(e) = client.send_to(data, client_addr) {
                                    println!("ERROR [{}]: proxy->client send: {}", client_addr, e);
                                }
                            }
                            Err(e) => {
                                if e.kind() != io::ErrorKind::WouldBlock {
                                    if e.kind() == io::ErrorKind::TimedOut {
                                        /*
                                            TODO: Sometimes peers don't get timed out.
                                        */
                                        if let Err(e) = tx.send(client_addr) {
                                            println!("ERROR [{}]: could not send a timed out notification: {}", client_addr, e);
                                        }
                                        break;
                                    }
                                    println!("ERROR [{}]: proxy<-server recv: {}", client_addr, e);
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    /*
                        TODO: ConnectionReset.
                    */
                    if e.kind() != io::ErrorKind::TimedOut && e.kind() != io::ErrorKind::ConnectionReset {
                        println!("ERROR: proxy<-client recv: {}", e);
                    }
                }
            }
        }

        let time = Instant::now();

        if time.duration_since(last_cleanup) >= CLEANUP_PERIOD_SECS {
            for client_addr in rx.try_iter() {
                println!("TIMEDOUT {}", client_addr);
                clients.remove(&client_addr);
            }
            last_cleanup = time;
        }
    }
}

fn buf_correct_port(buf: &mut [u8], port: u16) {
    buf[OFFSET_TO_PORT..][..2].copy_from_slice(&port.to_le_bytes());
}

fn buf_insert_name_suffix(buf: &mut [u8], len: &mut usize) {
    if SUFFIX.len() <= buf.len() - *len {
        let Some(pos) = buf[OFFSET_TO_NAME..].iter().position(|&b| b == 0)
        else {
            return;
        };
        let s = OFFSET_TO_NAME + pos;
        let e = s + SUFFIX.len();
        buf.copy_within(s..*len, e);
        buf[s..e].copy_from_slice(SUFFIX);
        buf[OFFSET_TO_NAME] += SUFFIX.len() as u8;
        *len += SUFFIX.len();
    }
}
