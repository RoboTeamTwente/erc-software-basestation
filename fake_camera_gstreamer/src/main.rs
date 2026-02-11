use anyhow::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize GStreamer
    gst::init()?;

    // Helper function to create a pipeline
    fn create_pipeline(pattern: &str, port: u32) -> Result<gst::Pipeline> {
        let pipeline_str = format!(
            "videotestsrc is-live=true pattern={} \
            ! video/x-raw,width=640,height=480,framerate=30/1 \
            ! videoconvert \
            ! x264enc tune=zerolatency speed-preset=ultrafast bitrate=2000 key-int-max=30 \
            ! rtph264pay config-interval=1 pt=96 \
            ! udpsink host=127.0.0.1 port={} sync=false",
            pattern, port
        );

        let pipeline = gst::parse::launch(&pipeline_str)?
            .downcast::<gst::Pipeline>()
            .expect("Expected a Pipeline");

        pipeline.set_state(gst::State::Playing)?;
        Ok(pipeline)
    }

    // Create 3 pipelines with different patterns and ports
    let pipelines = vec![
        create_pipeline("ball", 4500)?,
        create_pipeline("smpte", 4501)?,
        create_pipeline("snow", 4502)?,
    ];

    println!("Fake cameras streaming to UDP ports 4500, 4501, 4502");

    // Keep process alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
