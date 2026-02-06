use anyhow::Result;
use bytes::Bytes;
use std::convert::Infallible;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app::{AppSink, AppSinkCallbacks};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use async_stream::stream;


// Shared buffer to store JPEG frames from GStreamer
type FrameBuffer = Arc<Mutex<Option<Bytes>>>;


pub async fn start_http_server(frames: FrameBuffer, port: u16) {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["GET"]);

    let route = warp::path::end()
        .and_then({
            let frames = frames.clone();
            move || {
                let frames = frames.clone();

                async move {
                    let mjpeg_stream = stream! {
                        loop {
                            let maybe_frame = {
                                let guard = frames.lock().await;
                                guard.clone()
                            };

                            if let Some(frame) = maybe_frame {
                                let header = format!(
                                    "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                                    frame.len()
                                );

                                yield Ok::<Bytes, Infallible>(Bytes::from(header));
                                yield Ok::<Bytes, Infallible>(frame);
                                yield Ok::<Bytes, Infallible>(Bytes::from("\r\n"));

                            }

                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                    };

                    let body = warp::hyper::Body::wrap_stream(mjpeg_stream);

                    Ok::<_, Infallible>(
                        warp::http::Response::builder()
                            .header(
                                "Content-Type",
                                "multipart/x-mixed-replace; boundary=frame",
                            )
                            .body(body)
                            .unwrap()
                    )
                }
            }
        })
        .with(cors);

    println!("MJPEG HTTP server running on http://0.0.0.0:{}", port);

    warp::serve(route)
        .run(([0, 0, 0, 0], port))
        .await;
}

/// Start a GStreamer pipeline for a given UDP port and store JPEG frames
fn start_pipeline(port: u16, frames: FrameBuffer) -> Result<()> {
    let pipeline = gst::parse::launch(&format!(
        "udpsrc port={} caps=\"application/x-rtp,media=video,encoding-name=H264,payload=96\" \
         ! rtpjitterbuffer latency=50 \
         ! rtph264depay \
         ! avdec_h264 \
         ! videoconvert \
         ! jpegenc \
         ! appsink name=appsink",
        port
    ))?
    .downcast::<gst::Pipeline>()
    .unwrap();

    let appsink = pipeline
        .by_name("appsink")
        .unwrap()
        .downcast::<AppSink>()
        .unwrap();

    let frames_clone = frames.clone();
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = match sink.pull_sample() {
                    Ok(s) => s,
                    Err(_) => return Ok(gst::FlowSuccess::Ok),
                };

                let buffer = match sample.buffer() {
                    Some(b) => b,
                    None => return Ok(gst::FlowSuccess::Ok),
                };

                let map = buffer.map_readable().unwrap();
                let frame = Bytes::copy_from_slice(map.as_slice());

                // Update the shared frame buffer asynchronously
                let frames_clone = frames_clone.clone();
                tauri::async_runtime::spawn(async move {
                    *frames_clone.lock().await = Some(frame);
                });

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_state(gst::State::Playing)?;
    println!("Receiving H264 stream on udp://0.0.0.0:{}", port);
    Ok(())
}

pub async fn stream() -> Result<()> {
    gst::init()?;

    // Two frame buffers for two cameras
    let frames1: FrameBuffer = Arc::new(Mutex::new(None));
    //let frames2: FrameBuffer = Arc::new(Mutex::new(None));

    // Start HTTP servers for both streams
    {
        let frames1_clone = frames1.clone();
        tauri::async_runtime::spawn(async move {
            start_http_server(frames1_clone, 5000).await;
        });

        // let frames2_clone = frames2.clone();
        // tauri::async_runtime::spawn(async move {
        //     start_http_server(frames2_clone, 5001).await;
        // });
    }

    // Start GStreamer pipelines
    start_pipeline(4500, frames1.clone())?;
    //start_pipeline(4501, frames2.clone())?;

    // Keep alive
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}