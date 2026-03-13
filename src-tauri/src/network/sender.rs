use tokio::net::UdpSocket;                                                                               
use prost::Message;

use crate::proto::packets::*;                                                                                      
                                              

  pub async fn send_ping(
      socket: &UdpSocket,
      addr: &str,
      msg: crate::proto::packets::SensorBoardImuInfo,
  ) -> anyhow::Result<()> {
    let envelope = crate::proto::packets::PbEnvelope {
      payload: Some(pb_envelope::Payload::ImuInfo(msg))
    };

      let mut payload = Vec::new();
      envelope.encode(&mut payload)?;

      socket.send_to(&payload, addr).await?;

      Ok(())
  }