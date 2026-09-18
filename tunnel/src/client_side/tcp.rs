
use crate::prelude::*;

pub fn main(client_port: u16, server_port: Option<u16>)
{
    let Ok(client) = TcpListener::bind(("0.0.0.0", client_port)).map_err(|e| {
        println_err!("{}", e);
    })
    else {
        return;
    };

    let server = {
        let Ok(socket) = Socket::new(Domain::IPV4, Type::STREAM, Protocol::TCP.into()).map_err(|e| {
            println_err!("{}", e);
        })
        else {
            return;
        };

        let socket_addr = SocketAddr::from(([0; 4], server_port.unwrap_or(0)));

        socket.set_reuse_address(true).unwrap();
        socket.set_linger(Duration::ZERO.into()).unwrap();
        socket.set_tcp_nodelay(true).unwrap();
        socket.set_tcp_keepalive(&{
            let secs = Duration::from_secs(5);
            TcpKeepalive::new()
              . with_interval(secs)
              . with_time(secs)
        }).unwrap();
        socket.bind(&socket_addr.into()).unwrap();
        socket.listen(32).unwrap();

        TcpListener::from(socket)
    };

    println!("Client-Side Port: {}", client_port);
    println!("Server-Side Addr: {}", server.local_addr().unwrap());

    let c = Listen {
        server,
        server_stream: None.into(),
        client,
        client_stream: None.into(),
    };

    std::thread::scope(|s| {
        s.spawn(|| listen_client(&c));
        s.spawn(|| listen_server(&c));
    })
}

struct Listen {
    server: TcpListener,
    server_stream: RwLock<Option<TcpStream>>,
    client: TcpListener,
    client_stream: RwLock<Option<TcpStream>>,
}

fn listen_client(c: &Listen)
{
    let ref mut buf = vec![0; 65536];

    for incoming in c.client.incoming()
    {
        let mut client = match incoming {
            Ok (__) => __,
            Err (e) => {
                println_err!("{}", e);
                continue;
            }
        };

        println!("Client Connected {}.", client.peer_addr().unwrap());

        *c.client_stream.write().unwrap() = client.try_clone().unwrap().into();

        loop {
            let data = match client.read(buf) {
                Ok  (0) => break,
                Ok  (n) => &buf[..n],
                Err (e) => {
                    if e.kind() != ConnectionReset {
                        println_err!("{}", e);
                    }
                    continue;
                }
            };

            if
                let Ok  (mut server_stream) = c.server_stream.try_write() &&
                let Some(server) = &mut *server_stream
            {
                if let Err(e) = server.write_all(data) && e.kind() != ErrorKind::ConnectionReset {
                    println_err!("{}", e);
                }
            }
        }

        println!("Client Disonnected.");
    }
}

fn listen_server(c: &Listen)
{
    let ref mut buf = vec![0; 65536];

    for incoming in c.server.incoming()
    {
        let mut server = match incoming {
            Ok (__) => __,
            Err (e) => {
                println_err!("{}", e);
                continue;
            }
        };

        println!("Server Connected.");

        *c.server_stream.write().unwrap() = server.try_clone().unwrap().into();

        loop {
            let data = match server.read(buf) {
                Ok  (0) => break,
                Ok  (n) => &buf[..n],
                Err (e) => {
                    if e.kind() != ConnectionReset {
                        println_err!("{}", e);
                    }
                    continue;
                }
            };

            if
                let Ok  (mut client_stream) = c.client_stream.try_write() &&
                let Some(client) = &mut *client_stream
            {
                if let Err(e) = client.write_all(data) && e.kind() != ErrorKind::ConnectionReset {
                    println_err!("{}", e);
                }
            }
        }

        println!("Server Disonnected.");
    }
}
