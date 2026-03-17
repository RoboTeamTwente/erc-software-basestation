use tokio::net::UdpSocket;                                                                               
use prost::Message;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::net::ToSocketAddrs;

use crate::proto::packets::*;                                                                                      
                                              
pub async fn send_ping(socket: &UdpSocket, addr: &str, msg: crate::proto::packets::SensorBoardImuInfo,) -> anyhow::Result<()> {
    let envelope = crate::proto::packets::PbEnvelope {
      payload: Some(pb_envelope::Payload::ImuInfo(msg))
    };

    let mut payload = Vec::new();
    envelope.encode(&mut payload)?;

    socket.send_to(&payload, addr).await?;

    Ok(())
}

pub fn stream_dummy_imu_blocking(socket: Arc<UdpSocket>, addr: String, cancel: Arc<Mutex<bool>>) {
    // Resolve the address string once up front into a SocketAddr
    let socket_addr = addr
        .to_socket_addrs()
        .expect("Invalid address")
        .next()
        .expect("Could not resolve address");
 
    let mut t: f32 = 0.0;
    let dt: f32 = 0.05;
    let mut heading: f32 = 0.0;
    let mut n = [0f32; 8];
    let mut seed: u32 = 0xdeadbeef;
 
    let raw_noise = |s: &mut u32| -> f32 {
        *s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        (*s as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    let alpha: f32 = 0.05;
 
    loop {
        std::thread::sleep(Duration::from_millis(100));
 
        if *cancel.lock().unwrap() {
            println!("Dummy IMU stream exiting");
            break;
        }
 
        t += dt;
 
        for v in n.iter_mut() {
            *v += alpha * (raw_noise(&mut seed) - *v);
        }
 
        heading = (t * 0.008).sin() * 0.35;
        let yaw_rate_degs = (t * 0.008).cos() * 0.35 * 0.008_f32.to_degrees() * 0.05;
        let heave = (t * 0.3).sin() * 0.08;
 
        let accel_x = (t * 0.10).sin() * 0.05 + n[0] * 0.03;
        let accel_y = n[1] * 0.02;
        let accel_z = 9.81 + heave + n[2] * 0.04;
 
        let gyro_x = (t * 0.25).sin() * 0.8 + n[3] * 0.2;
        let gyro_y = (t * 0.18).cos() * 0.5 + n[4] * 0.2;
        let gyro_z = yaw_rate_degs           + n[5] * 0.1;
 
        let earth_h: f32 = 20.0;
        let earth_v: f32 = 42.0;
        let mag_x = earth_h * heading.cos() + n[6] * 0.3;
        let mag_y = earth_h * heading.sin() + n[6] * 0.3;
        let mag_z = earth_v                 + n[7] * 0.2;
 
        let msg = SensorBoardImuInfo {
            accel_x, accel_y, accel_z,
            gyro_x, gyro_y, gyro_z,
            mag_x, mag_y, mag_z,
            is_calibrated: true,
            state: SensorState::SensorOperating as i32,
            error_code: ImuErrorCode::ImuNoError as i32,
        };
 
        let envelope = PbEnvelope {
            payload: Some(pb_envelope::Payload::ImuInfo(msg)),
        };
        let mut buf = Vec::new();
        if envelope.encode(&mut buf).is_err() {
            eprintln!("Failed to encode dummy IMU packet");
            continue;
        }
        // try_send_to takes a SocketAddr, not a &str
        if let Err(e) = socket.try_send_to(&buf, socket_addr) {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                eprintln!("Failed to send dummy IMU packet: {e}");
            }
        }
    }
}