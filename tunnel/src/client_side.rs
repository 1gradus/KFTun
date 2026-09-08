
use crate::prelude::*;

pub fn main(client_port: u16, server_port: Option<u16>)
{
    // client-side port.
    // messages from/to client are sent/received here.
    let Ok(client) = UdpSocket::bind(("0.0.0.0", client_port)).map_err(|e| {
        println_err!("{}", e);
    })
    else {
        return;
    };

    // server-side port.
    // messages from/to server are sent/received here.
    let server = {
        use socket2::*;

        let Ok(socket) = Socket::new(Domain::IPV4, Type::DGRAM, Protocol::UDP.into()).map_err(|e| {
            println_err!("{}", e);
        })
        else {
            return;
        };

        let socket_addr = SocketAddr::from(([0; 4], server_port.unwrap_or(0)));

        socket.set_reuse_address(true).unwrap();
        socket.bind(&socket_addr.into()).unwrap();

        UdpSocket::from(socket)
    };

    set_report_reset(&client, false).unwrap();
    set_report_reset(&server, false).unwrap();

    // @TODO: STUN for server-side port.

    println!("Server Port: {}", client_port);
    println!("Client Addr: {}", server.local_addr().unwrap());

    let c = Listen {
        server_addr: None.into(),
        server,
        client,
        client_ports: Map::new().into(),
    };

    std::thread::scope(|s| {
        s.spawn(|| listen_client(&c));
        s.spawn(|| listen_server(&c));
    });
}

struct Listen {
    server_addr: RwLock<Option<SocketAddr>>,
    server: UdpSocket,
    client: UdpSocket,
    client_ports: RwLock<Map<u16, IpAddr>>,
}

fn listen_client(c: &Listen)
{
    let ref mut buf = vec![0; 65536];
    let ref mut msg_buf = vec![0; 65536];

    loop {
        let Ok((data, peer_addr)) = c.client.recv_from(buf)
          . map(|(n, a)| (&buf[..n], a))
          . map_err(|e| {
                println_err!("recv from client: {}", e);
            })
        else {
            continue;
        };

        c.client_ports.write().unwrap()
          . entry(peer_addr.port())
          . insert_entry(peer_addr.ip());

        if let Some(server_addr) = *c.server_addr.read().unwrap() {
            if let Err(e) = c.server.send_to(msg_data(msg_buf, peer_addr.port(), data), server_addr) {
                println_err!("send to server: {}", e);
            }
        }
    }
}

fn listen_server(c: &Listen)
{
    let ref mut buf = vec![0; 65536];

    loop {
        let Ok((msg, peer_addr)) = c.server.recv_from(buf)
          . map(|(n, a)| (&buf[..n], a))
          . map_err(|e| {
                println_err!("recv from server: {}", e)
            })
        else {
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
                c.server.send_to(msg_announce(buf), peer_addr).unwrap();
                c.server_addr.write().map(|mut addr| *addr = peer_addr.into()).unwrap();
            }
            Message::Echo => {
                println!("[ECHO] {}", peer_addr);
                c.server.send_to(msg_echo(buf), peer_addr).unwrap();
            }
            Message::Data (port, data) => {
                println!("[DATA] {} bytes from {} [{}]", data.len(), peer_addr, port);

                if let Some(&ip) = c.client_ports.read().unwrap().get(&port)
                {
                    if let Err(e) = c.client.send_to(data, (ip, port)) {
                        println_err!("send to client: {}", e);
                    }
                }
            }
        }
    }
}
