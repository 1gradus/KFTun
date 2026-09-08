
/*
    [      ] <-----> [            ]         [            ] <-----> [      ]
    [server]         [client-proxy] <-----> [server-proxy]         [client]
    [      ] <-----> [            ]         [            ] <-----> [      ]
*/

mod prelude {
    pub(crate) use crate::protocol::*;
    pub(crate) use std::{
        collections::{
            hash_map::{
                Entry,
            },
            HashMap as Map,
        },
        net::{
            IpAddr,
            SocketAddr,
            UdpSocket,
        },
        time::{
            Duration,
        },
        sync::{
            RwLock,
        },
        io::{
            ErrorKind,
        },
        os::windows::io::{
            AsRawSocket,
        },
    };

    pub(crate) type Result<T = (), E = std::io::Error> = ::core::result::Result<T, E>;

    #[link(name = "ws2_32.dll", kind = "raw-dylib")]
    unsafe extern "system"
    {
        safe fn ioctlsocket(
            socket: std::os::windows::raw::SOCKET,
            cmd: u32,
            arg: *mut u32,
        ) -> i32;
    }

    const SIO_UDP_CONNRESET: u32 = 2550136844;
    const SIO_UDP_NETRESET : u32 = 2550136847;

    pub(crate) fn set_report_reset(socket: &UdpSocket, flag: bool) -> Result
    {
        match ioctlsocket(socket.as_raw_socket(), SIO_UDP_CONNRESET, &mut (flag as u32)) {
            0 => {}
            e => return Err(std::io::Error::from_raw_os_error(e)),
        }
        match ioctlsocket(socket.as_raw_socket(), SIO_UDP_NETRESET, &mut (flag as u32)) {
            0 => {}
            e => return Err(std::io::Error::from_raw_os_error(e)),
        }
        Ok(())
    }
}

#[macro_use]
mod macros;

mod protocol;

mod server_side;
mod client_side;

fn main()
{
    let Some(CommandLine {
        side,
        port,
        client_addr,
        server_port,
    }) = command_line()
    else {
        return;
    };

    let Ok(port) = port.parse()
    else {
        println!("ERROR: port must be a number in range 0-65535, but it's '{}'", port);
        return
    };

    let client_addr = if let Some(addr) = client_addr {
        let Ok(client_addr) = addr.parse()
        else {
            println!("ERROR: client-addr must be an address in the format of IP:PORT, but it's '{}'", addr);
            return;
        };
        Some(client_addr)
    } else {
        None
    };

    let server_port = if let Some(port) = server_port {
        let Ok(client_port) = port.parse()
        else {
            println!("ERROR: server-port must be a number in range 0-65535, but it's '{}'", port);
            return;
        };
        Some(client_port)
    } else {
        None
    };

    match side {
        Side::Server => server_side::main(port, client_addr.unwrap()),
        Side::Client => client_side::main(port, server_port),
    }
}

const HELP_MESSAGE: &str = concat![
    "USAGE:\n",
    "    tun server <server-port> <client-addr>\n",
    "    tun client <client-port> [server-port]",
];

struct CommandLine {
    side: Side,
    port: String,
    client_addr: Option<String>,
    server_port: Option<String>,
}

#[derive(Copy, Clone)]
enum Side {
    Server,
    Client,
}

fn command_line() -> Option<CommandLine>
{
    let mut args = std::env::args();
    let _ = args.next();

    let mut help = false;
    let mut side = None;
    let mut n = 0;
    let mut port = None;
    let mut client_addr = None;
    let mut server_port = None;

    while let Some(arg) = args.next()
    {
        match arg.as_str()
        {
            "-h" | "--help" => help = true,
            "server" => side = Side::Server.into(),
            "client" => side = Side::Client.into(),
            _ if side.is_none() => {
                panic!("invalid command-line");
            }
            _ => {
                match n {
                    0 => port = arg.into(),
                    1 => {
                        match side.unwrap() {
                            Side::Server => client_addr = arg.into(),
                            Side::Client => server_port = arg.into(),
                        }
                    },
                    _ => panic!("invalid command-line"),
                }
                n += 1;
            },
        }
    }

    help |=
        matches!(&side, None) ||
        matches!(&side, Some(Side::Server)) && (port.is_none() || client_addr.is_none()) ||
        matches!(&side, Some(Side::Client)) && (port.is_none());

    if help {
        println!("{}", HELP_MESSAGE);
        return None;
    }

    Some (CommandLine {
        side: side.unwrap(),
        port: port.unwrap(),
        client_addr,
        server_port,
    })
}
