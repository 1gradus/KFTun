
pub mod server_side;
pub mod client_side;

mod prelude {
    pub(crate) use crate::prelude::{
        SocketAddr,
    };
    pub(crate) use std::sync::mpsc::{
        Sender,
        Receiver,
        channel,
    };

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
