use bytes::{BufMut, BytesMut};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = TcpStream::connect("localhost:8000").await?;
    let message = format_args!("hello server, i am {}", conn.local_addr()?).to_string();
    let mut buf = BytesMut::new();
    for x in 0..=20 {
        buf.clear();
        buf.put(&message.as_bytes()[..]);
        let msg = format_args!("{}, this idx {} message",&message,&x).to_string();
        buf.put(&msg.as_bytes()[..]);
        conn.write_all(&buf).await?;
        println!("send message {} size",buf.len());
        buf.clear();
        let n = conn.read_buf(&mut buf).await?;
        println!("receive {} length message: {:?}",buf.len(),&buf[..n]);
    }

    drop(conn);
    println!("close connection");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    println!("exit");
    Ok(())
}