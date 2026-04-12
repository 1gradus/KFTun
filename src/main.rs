
use std::net::{
    SocketAddr,
};
use std::process::{
    ExitCode,
};

mod server;
use server::{
    ServerCfg,
    server,
};

fn main() -> ExitCode {
    let [listen_addr, target_addr] = match command_line() {
        Ok(__) => __,
        Err(e) => return e,
    };
    server(ServerCfg {
        listen_addr,
        target_addr,
    })
}

const HELP_MESSAGE: &str = concat![
    "USAGE:\r\n",
    "    server <listen-address> <target-address>\r\n",
    "\r\n",
    "PARAMETERS:\r\n",
    "    <listen-address>  An address to which the proxy server will bind.\r\n",
    "    <target-address>  An address of the actual server.\r\n",
    "\r\n",
    "EXAMPLES:\r\n",
    "    Bind to all available interfaces:\r\n",
    "        server 0.0.0.0:7707 12.34.56.78:7707\r\n",
    "    Bind to a specific address:\r\n",
    "        server 127.0.0.1:7707 12.34.56.78:7707\r\n",
    "        server 100.1.2.3:7707 12.34.56.78:7707",
];

fn command_line() -> Result<[SocketAddr; 2], ExitCode> {
    let mut args = std::env::args();
    let _ = args.next();

    let mut listen_addr = None;
    let mut target_addr = None;
    let mut error = false;

    for arg in args {
        if listen_addr.is_none() {
            listen_addr = arg.parse().map_err(|_| {
                println!("ERROR: '{}' is not an acceptable listen address", arg);
                error = true;
            }).into();
        } else if target_addr.is_none() {
            target_addr = arg.parse().map_err(|_| {
                println!("ERROR: '{}' is not an acceptable target address", arg);
                error = true;
            }).into();
        } else {
            println!("ERROR: unexpected argument '{}'", arg);
            error = true;
        }
    }

    if error {
        println!("----------------------------------------");
        println!("{}", HELP_MESSAGE);
        return Err(ExitCode::FAILURE);
    }

    let (Some(Ok(listen_addr)), Some(Ok(target_addr))) = (listen_addr, target_addr)
    else {
        println!("{}", HELP_MESSAGE);
        return Err(ExitCode::SUCCESS);
    };

    Ok([
        listen_addr,
        target_addr,
    ])
}
