use bytes::BytesMut;
use chrono::{DateTime, Local};
use log::{error, info};
use std::net::SocketAddr;
use tokio::{
    io::AsyncReadExt,
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio::{
    io::AsyncWriteExt,
    sync::broadcast::{Receiver as M_Receiver, Sender as M_Sender},
};
use uuid::Uuid;

use super::packet::Packet;

#[derive(Debug)]
pub enum NetModel {
    Client,
    Stream,
}
pub struct ConnStats {
    pub id: u64,
    pub rs: OwnedReadHalf,
    pub ws: OwnedWriteHalf,
    pub peer: SocketAddr,
    pub read_buf: BytesMut,
    pub chat_send_ch: M_Sender<Packet>,
    pub chat_recv_ch: M_Receiver<Packet>,
    pub write_buf: BytesMut,
    pub nickname: String,
    pub login_time: DateTime<Local>,
    pub net_model: Option<NetModel>,
}

impl ConnStats {
    pub fn with_param(
        rs: OwnedReadHalf,
        ws: OwnedWriteHalf,
        peer: SocketAddr,
        ch: M_Sender<Packet>,
    ) -> ConnStats {
        let rc = ch.subscribe();
        ConnStats {
            id: Local::now().timestamp_nanos() as u64,
            rs,
            ws,
            peer,
            chat_send_ch: ch,
            chat_recv_ch: rc,
            read_buf: BytesMut::with_capacity(4096),
            write_buf: BytesMut::with_capacity(4096),
            nickname: Uuid::new_v4().to_string(),
            login_time: Local::now(),
            net_model: None,
        }
    }

    #[inline]
    pub async fn process(mut self) {
        if let Err(e) = self.ws.write_all(b"\r\nplease set your name: \r\n").await {
            error!("failed to send init message. err: {}", e);
            return;
        }
        loop {
            match self.rs.read_buf(&mut self.read_buf).await {
                Ok(n) => {
                    if n == 0 {
                        // close
                        return;
                    }
                    // 数据长度超过128认为错误
                    if self.read_buf.len() > 128 {
                        self.read_buf.clear();
                        if let Err(e) = self.ws.write_all(b"retry set your name: \r\n").await {
                            error!("failed to send reset username message. err: {}", e);
                            return;
                        }
                        continue;
                    }
                    if self.read_buf.len() < 2 {
                        continue;
                    }
                    let msg_len = self.read_buf.len();
                    if self.read_buf[msg_len - 2] != b'\r' && self.read_buf[msg_len - 1] != b'\n' {
                        continue;
                    }
                    if self.read_buf.len() - 2 < 1 {
                        self.read_buf.clear();
                        if let Err(e) = self
                            .ws
                            .write_all(b"Invalid username. retry set your name: \r\n")
                            .await
                        {
                            error!("failed to send reset username message. err: {}", e);
                            return;
                        }
                        continue;
                    }
                    self.read_buf.truncate(self.read_buf.len() - 2);
                    let data: Vec<u8>;
                    if self.read_buf.len() > 4
                        && self.read_buf.starts_with(&[0x78, 0x56, 0x56, 0x78])
                    {
                        // prefix len 4
                        self.net_model = Some(NetModel::Client);
                        data = self.read_buf.split_off(4).to_vec();
                        self.read_buf.clear();
                    } else {
                        self.net_model = Some(NetModel::Stream);
                        data = self.read_buf.split().to_vec();
                    }
                    match String::from_utf8(data) {
                        Ok(nickname) => {
                            self.nickname = nickname;
                        }
                        Err(_) => {
                            if let Err(e) = self
                                .ws
                                .write_all(b"Invalid UTF-8 sequence, retry set your name: \r\n")
                                .await
                            {
                                error!("failed to send reset username message. err: {}", e);
                                return;
                            }
                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("failed to read init message. err: {}", e);
                    return;
                }
            }
            if let Err(e) = self.ws.write_all(b"success! welcome \r\n").await {
                error!("failed to write message. err: {}", e);
                return;
            }
            break;
        }
        loop {
            tokio::select! {
                m = self.chat_recv_ch.recv() => {
                    match m {
                        Ok(m) => {
                            info!("receive global message: {:?}",&m.msg);
                            if m.user_id == self.id {
                                continue;
                            }
                            // write message
                            match self.net_model {
                                Some(NetModel::Stream) => {
                                    let mut mx = String::with_capacity(100);
                                    mx.push_str("[世界] [");
                                    mx.push_str(&m.get_time_format());
                                    mx.push_str("]: ");

                                    if let Err(e) = self.ws.write_all(&format!("[世界] [{}] [{}]: {}\r\n",m.get_time_format(),&m.username.unwrap(),&m.msg.unwrap()).as_bytes()).await {
                                        error!("failed to send reset username message. err: {}", e);
                                        return;
                                    }
                                },
                                Some(NetModel::Client) => {

                                },
                                _ => return,
                            }
                        }
                        Err(e) => error!("failed to get new message from receive channel. err: {}", e),
                    }
                }
                x = self.rs.read_buf(&mut self.read_buf) => {
                    match x {
                        Ok(n) => {
                            if n == 0 {
                                info!("client {} close connection", &self.peer);
                                return;
                            }
                            // process
                            info!("receive message len: {}",n);
                            match self.net_model {
                                Some(NetModel::Stream) => match Packet::unmarshal_from_line(&mut self.read_buf) {
                                    Ok(p) => {
                                        for mut msg in p.into_iter() {
                                            msg.user_id = self.id;
                                            msg.username = Some(self.nickname.clone());
                                            if let Err(e) = self.chat_send_ch.send(msg){
                                                error!("failed to send global message, err: {}",e);
                                                return;
                                            }
                                            // info!("receive message {:?}", msg.msg);
                                        }
                                    }
                                    Err(e) => match e {
                                        super::packet::PacketError::NotEnoughBytes => {
                                            continue;
                                        }
                                        super::packet::PacketError::ProtocolError => return,
                                        _ => return,
                                    },
                                },
                                Some(NetModel::Client) =>{
                                    // unimp
                                    return;
                                },
                                _ => return,
                            }
                        }
                        Err(e) => {
                            error!("failed to read from addr: {}. with err: {:?}", &self.peer, &e);
                            return;
                        }
                    }
                }
            }
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn check_buf(&self) -> bool {
        if self.read_buf.is_empty() {
            return false;
        }
        return match self.net_model {
            Some(NetModel::Client) => Packet::check(&self.read_buf),
            Some(NetModel::Stream) => true,
            None => false,
        };
    }
}
