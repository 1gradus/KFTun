
pub mod server_side;
pub mod client_side;

mod prelude {
    pub(crate) use crate::prelude::*;
    pub(crate) use std::sync::mpsc::{
        Sender,
        Receiver,
        channel,
    };

    #[link(name = "ws2_32", kind = "raw-dylib")]
    unsafe extern "system"
    {
        safe fn ioctlsocket(
            socket: RawSocket,
            cmd: u32,
            arg: *mut u32,
        ) -> i32;
    }

    const SIO_UDP_CONNRESET: u32 = 2550136844;
    const SIO_UDP_NETRESET : u32 = 2550136847;

    pub(crate) fn set_report_reset(socket: &UdpSocket, flag: bool) -> Result
    {
        // Controls whether PORT_UNREACHABLE messages are reported.
        match ioctlsocket(socket.as_raw_socket(), SIO_UDP_CONNRESET, &mut (flag as u32)) {
            0 => {}
            e => return Err(std::io::Error::from_raw_os_error(e)),
        }
        // Controls whether NET_UNREACHABLE (TTL expired) messages are reported.
        match ioctlsocket(socket.as_raw_socket(), SIO_UDP_NETRESET, &mut (flag as u32)) {
            0 => {}
            e => return Err(std::io::Error::from_raw_os_error(e)),
        }
        Ok(())
    }

    pub(crate) enum LogMessage {
        Announce (SocketAddr),
        Echo (SocketAddr),
        Data (usize, SocketAddr, u16),
    }

    pub(crate) fn listen_log(log: Receiver<LogMessage>)
    {
        for message in log.iter()
        {
            match message
            {
                LogMessage::Announce (peer_addr) => {
                    println!("[ANNC] {}", peer_addr);
                }
                LogMessage::Echo (peer_addr) => {
                    println!("[ECHO] {}", peer_addr);
                }
                LogMessage::Data (data_len, peer_addr, port) => {
                    println!("[DATA] {} bytes from {} [{}]", data_len, peer_addr, port);
                }
            }
        }
    }
}
