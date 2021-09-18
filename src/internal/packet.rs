use bytes::{Buf, BytesMut};
use chrono::{DateTime, Local};
use log::{debug, info};
use snafu::{ensure, ResultExt, Snafu};

// message type (operation type)
#[derive(Debug,Clone)]
pub enum PacketType {
    Ping,
    Pong,
    Message,
    MessageAck,
}

/// message version
#[derive(Debug, Clone)]
pub enum PacketVersion {
    V1,
}

#[derive(Debug, Snafu)]
pub enum PacketError {
    #[snafu(display("not enough bytes to read"))]
    NotEnoughBytes,

    #[snafu(display("not has message"))]
    Null,

    #[snafu(display("protocol error"))]
    ProtocolError,

    #[snafu(display("unsupported protocol version"))]
    UnSupportVersion,

    #[snafu(display("failed to bytes transform to string"))]
    TransformFailed { source: std::string::FromUtf8Error },
}

/// Protocol
/// []byte[0,1] // message length
/// []byte[2] // protocol version

/// version 1
/// []byte[3] // control operation
/// []byte[4..] // message
#[derive(Clone)]
pub struct Packet {
    version: PacketVersion,
    operation: PacketType,
    id: u64,
    pub user_id: u64,
    pub username: Option<String>,
    // receive message time
    time: DateTime<Local>,
    pub msg: Option<String>,
}

impl PacketVersion {
    #[inline]
    fn from_u8(v: u8) -> Option<PacketVersion> {
        match v {
            1 => Some(PacketVersion::V1),
            _ => None,
        }
    }
}

impl PacketType {
    #[inline]
    fn from_u8(t: u8) -> Option<PacketType> {
        match t {
            1 => Some(PacketType::Ping),
            2 => Some(PacketType::Pong),
            3 => Some(PacketType::Message),
            4 => Some(PacketType::MessageAck),
            _ => None,
        }
    }

}

impl Packet {
    #[inline]
    pub fn new() -> Self {
        Packet {
            version: PacketVersion::V1,
            operation: PacketType::Message,
            id: 0,
            user_id: 0,
            username: None,
            time: Local::now(),
            msg: None,
        }
    }

    #[inline]
    pub fn set_user_profile(&mut self,uid: u64,username: String) {
        self.user_id = uid;
        self.username = Some(username);
    }
    
    #[inline]
    pub fn get_user_profile(&self) -> (u64,Option<String>) {
        (self.user_id, self.username.clone())
    }

    #[inline]
    pub fn unmarshal_from_byte(b: &mut BytesMut) -> std::result::Result<Vec<Self>, PacketError> {
        let mut msg_list = Vec::<Packet>::new();

        while b.len() > 2 {
            let header = b[..2].as_ref();
            let length = (header[0] as usize) << 8 + header[1];
            ensure!(length > 0, ProtocolError);
            // 消息大于 2^13 断开连接
            ensure!(length < 1 << 13, ProtocolError);
            if length < b.len() {
                break;
            }
            b.advance(2);
            let msg_buf = b.split_to(length);
            let mut msg = Packet::new();
            match PacketVersion::from_u8(msg_buf[0]) {
                Some(version) => match version {
                    PacketVersion::V1 => {
                        msg.version = PacketVersion::V1;
                        match PacketType::from_u8(msg_buf[1]) {
                            Some(operation) => {
                                msg.operation = operation;
                                let msg_str = String::from_utf8(msg_buf[2..].to_vec())
                                    .context(TransformFailed)?;
                                msg.msg = Some(msg_str);
                                msg_list.push(msg);
                            }
                            None => {}
                        }
                    }
                },
                None => return Err(PacketError::UnSupportVersion),
            };
        }
        Ok(msg_list)
    }
    
    #[inline]
    pub fn unmarshal_from_line(b: &mut BytesMut) -> std::result::Result<Vec<Self>, PacketError> {
        let mut idx = 1;
        let mut list = Vec::<Packet>::new();
        while b.len() > idx {
            if b[idx - 1] == b'\r' && b[idx] == b'\n' {
                let msg_data = b.split_to(idx-1);
                b.advance(2);
                if msg_data.len() == 0 {// 空数据
                    continue;
                }
                debug!("{:?}",&msg_data);
                let mut msg = Packet::new();
                msg.msg = Some(String::from_utf8_lossy(msg_data.as_ref()).to_string());
                list.push(msg);
                idx = 1;
            }
            idx += 1;
        }
        if list.len() == 0 {
            info!("no message return");
        }
        Ok(list)
    }

    #[inline]
    pub fn check(b: &BytesMut) -> bool {
        if b.len() < 2 {
            return false;
        }
        let length = (b[0] as usize) << 8 + b[1];
        if length < b.len() - 2 {
            return false;
        }
        true
    }

    #[inline]
    pub fn get_time_format(&self) -> String {
        return self.time.format("%H:%M:%S").to_string();
    }
}
