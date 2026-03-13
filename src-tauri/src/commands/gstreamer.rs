use anyhow::Result;
use bytes::Bytes;
use std::convert::Infallible;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app::{AppSink, AppSinkCallbacks};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use warp::Filter;
use async_stream::stream;
use tauri::Emitter;

// Shared buffer to store JPEG frames from GStreamer
type FrameBuffer = Arc<Mutex<Option<Bytes>>>;
// Tracks the last time a frame was received
type LastFrameTime = Arc<Mutex<Option<Instant>>>;

// Make this globally accessible via Tauri state
pub struct CameraHealth {
    pub stale: Arc<Mutex<Vec<(u16, bool)>>>,
}


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
fn start_pipeline(port: u16, frames: FrameBuffer, last_frame_time: LastFrameTime) -> Result<()> {
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
    let last_frame_time_clone = last_frame_time.clone();

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
                let last_frame_time_clone = last_frame_time_clone.clone();

                tauri::async_runtime::spawn(async move {
                    *frames_clone.lock().await = Some(frame);
                    *last_frame_time_clone.lock().await = Some(Instant::now());
                });

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_state(gst::State::Playing)?;
    println!("Receiving H264 stream on udp://0.0.0.0:{}", port);
    Ok(())
}


/// Watches all streams and emits Tauri events when feeds go stale
async fn watch_feed_health(
    app_handle: tauri::AppHandle,
    streams: Vec<(u16, LastFrameTime)>,
) {
    const STALE_THRESHOLD: Duration = Duration::from_secs(2);
    const POLL_INTERVAL: Duration = Duration::from_millis(500);

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        for (port, last_frame_time) in streams.iter() {
            let is_stale = {
                let guard = last_frame_time.lock().await;
                match *guard {
                    None => true,
                    Some(t) => t.elapsed() > STALE_THRESHOLD,
                }
            };

            // Always emit, not just on change — frontend may have missed earlier events
            let _ = app_handle.emit_to(
                tauri::EventTarget::any(),
                "camera-feed-status",
                serde_json::json!({
                    "port": port,
                    "stale": is_stale,
                }),
            );
        }
    }
}


pub async fn stream(app_handle: tauri::AppHandle) -> Result<()> {
    gst::init()?;

    // Two frame buffers for two cameras
    let frames1: FrameBuffer = Arc::new(Mutex::new(None));
    let frames2: FrameBuffer = Arc::new(Mutex::new(None));
    let frames3: FrameBuffer = Arc::new(Mutex::new(None));

    let last_frame1: LastFrameTime = Arc::new(Mutex::new(None));
    let last_frame2: LastFrameTime = Arc::new(Mutex::new(None));
    let last_frame3: LastFrameTime = Arc::new(Mutex::new(None));

    // Start HTTP servers for both streams
    {
        let frames1_clone = frames1.clone();
        tauri::async_runtime::spawn(async move {
            start_http_server(frames1_clone, 5000).await;
        });

        let frames2_clone = frames2.clone();
        tauri::async_runtime::spawn(async move {
            start_http_server(frames2_clone, 5001).await;
        });

        let frames3_clone = frames3.clone();
        tauri::async_runtime::spawn(async move {
            start_http_server(frames3_clone, 5002).await;
        });
    }

    // Start GStreamer pipelines
    start_pipeline(4500, frames1.clone(), last_frame1.clone())?;
    start_pipeline(4501, frames2.clone(), last_frame2.clone())?;
    start_pipeline(4502, frames3.clone(), last_frame3.clone())?;

    // Start health watcher — emits "camera-feed-status" events to frontend
    let streams = vec![
        (5000u16, last_frame1),
        (5001u16, last_frame2),
        (5002u16, last_frame3),
    ];
    tauri::async_runtime::spawn(watch_feed_health(app_handle, streams));

    // Keep alive
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}