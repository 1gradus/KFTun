
use crate::prelude::*;

pub fn main(server_port: u16, client_addr: SocketAddr)
{
    let server_addr = SocketAddr::from(([127, 0, 0, 1], server_port));

    println!("Server Addr: {}", server_addr);
    println!("Client Addr: {}", client_addr);

    let c = Listen {
        server_addr,
        server_stream: None.into(),
        client_addr,
        client_stream: None.into(),
    };

    std::thread::scope(|s| {
        s.spawn(|| stream_client(&c));
        s.spawn(|| stream_server(&c));
    });
}

struct Listen {
    server_addr: SocketAddr,
    server_stream: RwLock<Option<TcpStream>>,
    client_addr: SocketAddr,
    client_stream: RwLock<Option<TcpStream>>,
}

fn stream_server(c: &Listen)
{
    let ref mut buf = vec![0; 65536];

    loop {
        let Ok(mut server) = TcpStream::connect(c.server_addr).map_err(|e| {
            if e.kind() != ConnectionRefused {
                println_err!("{}", e);
            }
        })
        else {
            continue;
        };

        println!("Server Connected.");

        set_socket_options(&server).unwrap();

        *c.server_stream.write().unwrap() = server.try_clone().unwrap().into();

        loop {
            let data = match server.read(buf) {
                Ok  (0) => break,
                Ok  (n) => &buf[..n],
                Err (e) => {
                    if !matches!(e.kind(), ConnectionRefused | ConnectionReset) {
                        println_err!("{}", e);
                    }
                    break;
                }
            };

            if
                let Ok   (mut client_stream) = c.client_stream.try_write() &&
                let Some (client) = &mut *client_stream
            {
                if let Err(e) = client.write_all(data) && e.kind() != ConnectionReset {
                    println_err!("{}", e);
                }
            }
        }

        println!("Server Disconnected.");
    }
}

fn stream_client(c: &Listen)
{
    let ref mut buf = vec![0; 65536];

    loop {
        let Ok(mut client) = TcpStream::connect(c.client_addr).map_err(|e| {
            if e.kind() != ConnectionRefused {
                println_err!("{}", e);
            }
        })
        else {
            continue;
        };

        println!("Client Connected.");

        set_socket_options(&client).unwrap();

        *c.client_stream.write().unwrap() = client.try_clone().unwrap().into();

        loop {
            let data = match client.read(buf) {
                Ok  (0) => break,
                Ok  (n) => &buf[..n],
                Err (e) => {
                    if !matches!(e.kind(), ConnectionRefused | ConnectionReset) {
                        println_err!("{}", e);
                    }
                    break;
                }
            };

            if
                let Ok   (mut server_stream) = c.server_stream.try_write() &&
                let Some (server) = &mut *server_stream
            {
                if let Err(e) = server.write_all(data) && e.kind() != ConnectionReset {
                    println_err!("{}", e);
                }
            }
        }

        println!("Client Disconnected.");
    }

}

fn set_socket_options(socket: &TcpStream) -> Result<()>
{
    let socket = SockRef::from(socket);
    socket.set_tcp_nodelay(true)?;
    socket.set_linger(Duration::ZERO.into())?;
    socket.set_tcp_keepalive(&{
        let secs = Duration::from_secs(5);
        TcpKeepalive::new()
          . with_interval(secs)
          . with_time(secs)
    })?;
    Ok(())
}
