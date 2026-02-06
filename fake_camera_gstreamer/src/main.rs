use anyhow::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize GStreamer
    gst::init()?;

    // Fake camera pipeline
    let pipeline = gst::parse::launch(
        "videotestsrc is-live=true pattern=ball \
     ! video/x-raw,width=640,height=480,framerate=30/1 \
     ! videoconvert \
     ! x264enc tune=zerolatency speed-preset=ultrafast bitrate=2000 key-int-max=30 \
     ! rtph264pay config-interval=1 pt=96 \
     ! udpsink host=127.0.0.1 port=4500 sync=false",
    )?
    .downcast::<gst::Pipeline>()
    .expect("Expected Pipeline");

    pipeline.set_state(gst::State::Playing)?;

    println!("Fake H264 camera streaming to udp://127.0.0.1:4500");

    // Keep process alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
