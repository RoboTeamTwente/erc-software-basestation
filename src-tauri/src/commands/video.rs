use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use async_stream::stream;
use bytes::Bytes;
use hyper::{
    header, service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use image::{ImageOutputFormat, RgbImage};
use tokio::net::UdpSocket;

type SharedFrames = Arc<Mutex<HashMap<u16, Vec<u8>>>>;


fn yuyv_to_rgb(yuyv: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgb = vec![0u8; (width * height * 3) as usize];
    let mut i = 0;
    let mut j = 0;

    while i + 3 < yuyv.len() {
        let y0 = yuyv[i] as f32;
        let u  = yuyv[i + 1] as f32 - 128.0;
        let y1 = yuyv[i + 2] as f32;
        let v  = yuyv[i + 3] as f32 - 128.0;

        for &y in &[y0, y1] {
            let r = (y + 1.402 * v).clamp(0.0, 255.0);
            let g = (y - 0.344 * u - 0.714 * v).clamp(0.0, 255.0);
            let b = (y + 1.772 * u).clamp(0.0, 255.0);

            rgb[j] = r as u8;
            rgb[j + 1] = g as u8;
            rgb[j + 2] = b as u8;
            j += 3;
        }

        i += 4;
    }

    rgb
}

fn yuyv_to_jpeg(yuyv: &[u8], width: u32, height: u32) -> Vec<u8> {
    let rgb = yuyv_to_rgb(yuyv, width, height);
    let img = RgbImage::from_raw(width, height, rgb).unwrap();

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageOutputFormat::Jpeg(80)).unwrap();
    buf.into_inner()
}


async fn jpeg_listener(frames: SharedFrames) {
    let socket = UdpSocket::bind("0.0.0.0:5000").await.unwrap();
    let mut buf = vec![0u8; 65535];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await.unwrap();
        frames.lock().unwrap().insert(5000, buf[..len].to_vec());
    }
}


async fn yuyv_listener(frames: SharedFrames) {
    let socket = UdpSocket::bind("0.0.0.0:4500").await.unwrap();
    let width = 1920;  //--------------------------CHANGE WIDTH AND HEIGHT IF NEEDED --------------------------
    let height = 1080; //--------------------------CHANGE WIDTH AND HEIGHT IF NEEDED --------------------------
    let mut buf = vec![0u8; (width * height * 2) as usize];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await.unwrap();
        let jpeg = yuyv_to_jpeg(&buf[..len], width, height);
        frames.lock().unwrap().insert(4500, jpeg);
    }
}


pub async fn start_server() -> Result<()> {
    let frames: SharedFrames = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(jpeg_listener(frames.clone()));
    tokio::spawn(yuyv_listener(frames.clone()));

    let make_svc = make_service_fn(move |_| {
        let frames = frames.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let frames = frames.clone();
                async move {
                    match req.uri().path() {
                        p if p.starts_with("/snapshot/") => {
                            let port: u16 = p.split('/').last().unwrap().parse().unwrap();
                            let jpeg = frames.lock().unwrap().get(&port).cloned().unwrap_or_default();

                            Ok::<_, hyper::Error>(
                                Response::builder()
                                    .header(header::CONTENT_TYPE, "image/jpeg")
                                    .header(header::CONTENT_LENGTH, jpeg.len())
                                    .body(Body::from(jpeg))
                                    .unwrap(),
                            )
                        }

                        _ => Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("Not Found"))
                            .unwrap()),
                    }
                }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 8080).into();
    println!("Server running on http://{}", addr);
    Server::bind(&addr).serve(make_svc).await?;

    Ok(())
}



#[tauri::command]
pub async fn fetch_snapshot(port: u16) -> Result<Vec<u8>, String> {
    let url = format!("http://127.0.0.1:8080/snapshot/{}", port);
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    Ok(resp.bytes().await.map_err(|e| e.to_string())?.to_vec())
}
