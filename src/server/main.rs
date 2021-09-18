use log::{info, LevelFilter};
use std::{fs::read_to_string, net::SocketAddr};
use tokio::{
    net::{TcpListener, TcpStream,},
    sync::broadcast::Sender as M_Sender,
};

use mchat::{config::Config, internal::conn::ConnStats, CONFIG_FILENAME};
use mchat::internal::packet::Packet;
extern crate chrono;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf_path = read_to_string(CONFIG_FILENAME)?;
    let conf: Config = toml::from_str(&conf_path)?;
    log_init(&conf)?;

    let (group_chat_ch, _) = tokio::sync::broadcast::channel::<Packet>(5000);

    // tokio::spawn(handler_global_message(receiver,user_list.clone()));

    let listener = TcpListener::bind(&conf.server.addr).await?;
    info!("start service: {}", listener.local_addr()?);
    loop {
        let (conn, peer) = listener.accept().await?;
        info!("receive client connection from {}", &peer);
        tokio::spawn(handler_conn(conn, peer, group_chat_ch.clone()));
    }
}

async fn handler_conn(conn: TcpStream, peer: SocketAddr, send_ch: M_Sender<Packet>) {
    let (rs, ws) = conn.into_split();
    let stats = ConnStats::with_param(rs, ws, peer, send_ch);
    stats.process().await;
}

pub fn log_init(conf: &Config) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.level(),
                message
            ))
        })
        .level(conf.log.level.parse::<LevelFilter>().unwrap())
        .chain(std::io::stdout())
        .chain(fern::log_file(&conf.log.path)?)
        .apply()?;
    Ok(())
}
