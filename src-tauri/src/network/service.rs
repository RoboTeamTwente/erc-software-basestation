use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct UdpService {
    socket: Arc<UdpSocket>,
}

impl UdpService {
    pub async fn new(bind_addr: &str) -> anyhow::Result<Self> {
        let addr: SocketAddr = bind_addr.parse()?;
        
        let socket2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        println!("recv buf size: {}", socket2.recv_buffer_size()?);
        socket2.set_reuse_address(true)?;
        socket2.set_recv_buffer_size(4 * 1024 * 1024)?;
        socket2.set_nonblocking(true)?;
        socket2.bind(&addr.into())?;

        let std_sock: std::net::UdpSocket = socket2.into();
        let socket = UdpSocket::from_std(std_sock)?;

        Ok(Self {
            socket: Arc::new(socket),
        })
    }

    pub fn socket(&self) -> Arc<UdpSocket> {
        self.socket.clone()
    }
}