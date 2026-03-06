  use std::sync::atomic::{AtomicU32, Ordering};                                                            
  use tokio::net::UdpSocket;                                                                               
  use prost::Message;                                                                                      
                                                                                                           
  const MSG_TYPE_IMU: u16 = 1;                                                                             
  static SEQ: AtomicU32 = AtomicU32::new(0);

  pub async fn send_ping(
      socket: &UdpSocket,
      addr: &str,
      msg: crate::proto::packets::ImuSensorInformation,
  ) -> anyhow::Result<()> {
      let mut payload = Vec::new();
      msg.encode(&mut payload)?;

      let seq = SEQ.fetch_add(1, Ordering::Relaxed);

      let mut datagram = Vec::with_capacity(8 + payload.len());
      datagram.extend_from_slice(&MSG_TYPE_IMU.to_be_bytes());
      datagram.extend_from_slice(&(payload.len() as u16).to_be_bytes());
      datagram.extend_from_slice(&seq.to_be_bytes());
      datagram.extend_from_slice(&payload);

      socket.send_to(&datagram, addr).await?;

      Ok(())
  }