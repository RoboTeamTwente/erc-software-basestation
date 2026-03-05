use tokio::net::UdpSocket;
use prost::Message;

pub async fn run_listener(socket: std::sync::Arc<UdpSocket>) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 2048];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        match crate::proto::packets::Ping::decode(&buf[..len]) {
            Ok(msg) => {
                println!("Received from {addr}: {:?}", msg);
            }
            Err(e) => {
                eprintln!("Decode error: {e}");
            }
        }
    }
}