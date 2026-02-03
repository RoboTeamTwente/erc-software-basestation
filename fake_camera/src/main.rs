use std::net::UdpSocket;
use std::{thread, time};
use image::{ImageBuffer, Rgb};
use std::io::Cursor; // <-- add this

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target = "127.0.0.1:5000";

    let mut frame = 0u32;

    loop {
        // Generate fake image
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_fn(640, 480, |x, y| {
                let v = ((x + y + frame) % 255) as u8;
                Rgb([v, 100, 200])
            });

        // Use Cursor<Vec<u8>> instead of raw Vec<u8>
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageOutputFormat::Jpeg(80))?;

        // Send the inner Vec<u8> through UDP
        socket.send_to(&buf.into_inner(), target)?;
        frame += 1;

        thread::sleep(time::Duration::from_millis(33)); // ~30 FPS
    }
}
