  use std::sync::atomic::{AtomicU32, Ordering};                                                            
  use tokio::net::UdpSocket;                                                                               
  use prost::Message;

use crate::proto::packets::PbMessageType;                                                                                      
                                              

  pub async fn send_ping(
      socket: &UdpSocket,
      addr: &str,
      msg: crate::proto::packets::ImuSensorInformation,
  ) -> anyhow::Result<()> {
    let envelope = crate::proto::packets::PbMessageEnvelope {
        r#type: PbMessageType::SensorBoardImuInformation as i32,
        data: msg.encode_to_vec(),
    };

      let mut payload = Vec::new();
      envelope.encode(&mut payload)?;

      socket.send_to(&payload, addr).await?;

      Ok(())
  }