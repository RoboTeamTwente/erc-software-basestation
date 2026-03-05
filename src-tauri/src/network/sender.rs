use tokio::net::UdpSocket;
use prost::Message;

pub async fn send_ping(
    socket: &UdpSocket,
    addr: &str,
    msg: crate::proto::packets::ImuSensorInformation,
) -> anyhow::Result<()> {

    let mut buffer = Vec::new();
    msg.encode(&mut buffer)?;

    socket.send_to(&buffer, addr).await?;

    Ok(())
}