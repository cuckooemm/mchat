use std::{collections::HashMap, sync::Arc};

use tokio::{net::tcp::OwnedWriteHalf, sync::{Mutex, mpsc::Receiver}};
use crate::internal::packet::Packet;

#[allow(dead_code)]
pub async fn handler_global_message(mut _receive: Receiver<Packet>,_ulist: Arc<Mutex<HashMap<i64,OwnedWriteHalf>>>) {
    loop{
        // TODO
    }

}