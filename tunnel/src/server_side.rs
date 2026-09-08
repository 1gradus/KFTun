
/*
    1. Establish connection with the client(s).
    2. Begin Client/Server listening/tunneling.
*/

use crate::prelude::*;

pub fn main(server_port: u16, client_addr: SocketAddr)
{
    // @TODO: should be specified from command-line.
    let server_addr = SocketAddr::from(([127, 0, 0, 1], server_port));

    // client-side port.
    // messages from/to client are sent/received here.
    let Ok(client) = UdpSocket::bind("0.0.0.0:0").map_err(|e| {
        println_err!("{}", e);
    })
    else {
        return;
    };

    set_report_reset(&client, false).unwrap();

    println!("Server Addr: {}", server_addr);
    println!("Client Addr: {}", client_addr);

    let c = Listen {
        server_addr,
        server_ports: Map::new().into(),
        client_addr,
        client,
    };

    std::thread::scope(|s| {
        s.spawn(|| listen_client(&c));
        s.spawn(|| listen_server(&c));
    });
}

struct Listen {
    server_addr: SocketAddr,
    server_ports: RwLock<Map<u16, UdpSocket>>,
    client_addr: SocketAddr,
    client: UdpSocket,
}

fn listen_server(c: &Listen)
{
    let ref mut buf = vec![0; 65536];
    let ref mut msg_buf = vec![0; 65536];
    let mut sleep;

    loop {
        sleep = true;

        for (&port, socket) in c.server_ports.read().unwrap().iter()
        {
            let Ok((data, peer_addr)) = socket.recv_from(buf)
              . map(|(n, a)| (&buf[..n], a))
              . map_err(|e| {
                    if !matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) {
                        println_err!("recv from server: {}", e);
                    }
                })
            else {
                continue;
            };

            sleep = false;

            if c.server_addr != peer_addr {
                println_dbg!("unknown server peer {}", peer_addr);
                continue;
            }

            if let Err(e) = c.client.send_to(msg_data(msg_buf, port, data), c.client_addr) {
                println_err!("send to client: {}", e);
            }
        }

        if sleep {
            std::thread::sleep(Duration::from_micros(500));
        }
    }
}

fn listen_client(c: &Listen)
{
    let ref mut buf = vec![0; 65536];

    if let Err(e) = c.client.send_to(msg_announce(buf), c.client_addr) {
        println_err!("send announce: {}", e);
        return;
    }

    c.client.set_read_timeout(Duration::from_secs(5).into()).unwrap();

    loop {
        let Ok((msg, peer_addr)) = c.client.recv_from(buf)
          . map(|(n, a)| (&buf[..n], a))
          . map_err(|e| {
                if !matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) {
                    println_err!("recv from client: {}", e);
                }
            })
        else {
            if let Err(e) = c.client.send_to(msg_announce(buf), c.client_addr) {
                println_err!("send announce: {}", e);
            }
            continue;
        };

        let Some(msg) = msg_kind(msg)
        else {
            println_dbg!("invalid message from {}", peer_addr);
            continue;
        };

        match msg
        {
            Message::Announce => {
                println!("[ANNC] {}", peer_addr);
            }
            Message::Echo => {
                println!("[ECHO] {}", peer_addr);
            }
            Message::Data (port, data) => 'l: {
                println!("[DATA] {} bytes from {} [{}]", data.len(), peer_addr, port);

                let mut map = c.server_ports.write().unwrap();
                let socket = match map.entry(port) {
                    Entry::Occupied (e) => &*{ e.into_mut() },
                    Entry::Vacant   (e) => &*{
                        let Ok(socket) = UdpSocket::bind("0.0.0.0:0").map_err(|e| {
                            println_err!("{}", e);
                        })
                        else {
                            break 'l;
                        };

                        socket.set_nonblocking(true).unwrap();
                        set_report_reset(&socket, false).unwrap();

                        e.insert(socket)
                    },
                };

                if let Err(e) = socket.send_to(data, c.server_addr) {
                    println_err!("send to server: {}", e);
                }
            }
        }
    }
}
