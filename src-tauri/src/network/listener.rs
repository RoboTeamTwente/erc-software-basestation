use tokio::net::UdpSocket;
use prost::Message;

use bytes::{Buf, Bytes};  

const MSG_TYPE_IMU: u16 = 1;                                                                 
const HEADER_LEN: usize = 8; // PacketHeader: u16 + u16 + u32  

pub async fn run_listener(socket: std::sync::Arc<UdpSocket>) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 2048];
    
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        let payload = &buf[..len];

        match crate::proto::packets::ImuSensorInformation::decode(payload) {
            Ok(imu) => {
                println!("Received IMU from {addr}: {:?}", imu);
            }
            Err(e) => {
                eprintln!("IMU decode error from {addr}: {e}");
            }
        }

        // if len < HEADER_LEN { continue; } 
        // let mut hdr = &buf[..HEADER_LEN];                                                    
        // let msg_type     = hdr.get_u16();   // big-endian by default                         
        // let payload_len  = hdr.get_u16();                                                    
        // let _seq         = hdr.get_u32(); 


        // if len != HEADER_LEN + payload_len as usize { continue; } 
        // let payload = &buf[HEADER_LEN..len]; 
        
    //     match msg_type {                                                                     
    //             MSG_TYPE_IMU=> {                                                                                                                     
    //                 match crate::proto::packets::ImuSensorInformation::decode(payload) {                         
    //                     Ok(imu) => {
    //                         println!("Received IMU from {addr}: {:?}", imu);
    //                     }                                        
    //                     Err(e) => {
    //                         eprintln!("IMU decode error: {e}");
    //                     }                            
    //                 }                                                                            
    //             }, 0_u16 | 2_u16..=u16::MAX => todo!()                                                                                                                   
    //         }
    // }
    }
}