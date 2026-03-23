use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::net::ToSocketAddrs;
use prost::Message;
use tokio::net::UdpSocket;
use crate::proto::packets::*;

pub fn stream_dummy_imu_blocking(
    socket: Arc<UdpSocket>,
    addr: String,
    cancel: Arc<Mutex<bool>>,
) {
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

    let mut raw_noise = |s: &mut u32| -> f32 {
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

        let msg = SensorBoardImuInfo {
            accel_x: (t * 0.10).sin() * 0.05 + n[0] * 0.03,
            accel_y: n[1] * 0.02,
            accel_z: 9.81 + heave + n[2] * 0.04,
            gyro_x:  (t * 0.25).sin() * 0.8 + n[3] * 0.2,
            gyro_y:  (t * 0.18).cos() * 0.5 + n[4] * 0.2,
            gyro_z:  yaw_rate_degs          + n[5] * 0.1,
            mag_x:   20.0 * heading.cos()   + n[6] * 0.3,
            mag_y:   20.0 * heading.sin()   + n[6] * 0.3,
            mag_z:   42.0                   + n[7] * 0.2,
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

        if let Err(e) = socket.try_send_to(&buf, socket_addr) {
            if e.kind() != std::io::ErrorKind::WouldBlock {
                eprintln!("Failed to send dummy IMU packet: {e}");
            }
        }
    }
}