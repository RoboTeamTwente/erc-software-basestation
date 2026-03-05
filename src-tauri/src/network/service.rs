use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct UdpService {
    socket: Arc<UdpSocket>,
}

impl UdpService {
    pub async fn new(bind_addr: &str) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;

        Ok(Self {
            socket: Arc::new(socket),
        })
    }

    pub fn socket(&self) -> Arc<UdpSocket> {
        self.socket.clone()
    }
}