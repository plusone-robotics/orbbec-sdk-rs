//! Multi-device example demonstrating thread-safe concurrent device access
//! This is a Rust equivalent of the C++ multi-device example showing the SDK's thread safety

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use orbbec_sdk::{
    Context, SensorType,
    frame::FrameSet,
    pipeline::{Config, Pipeline},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Context using the global singleton - equivalent to ob::Context::globalInstance()
    let context = Context::global_instance(None)?;

    // Query the list of connected devices - equivalent to ctx.queryDeviceList()
    let device_list = context.query_device_list()?;

    // Get the number of connected devices - equivalent to devList->getCount()
    let device_count = device_list.len();
    
    println!("Found {} Orbbec devices", device_count);

    if device_count == 0 {
        println!("No devices found. Please connect at least one Orbbec device.");
        return Ok(());
    }

    // Create a pipeline for each device - equivalent to std::map<int, std::shared_ptr<ob::Pipeline>> pipes
    let mut pipelines = Vec::new();
    
    for i in 0..device_count {
        // Get the device from device list - equivalent to devList->getDevice(i)
        let device = device_list.get(i)?;
        
        // Get device info to access name and serial number
        let device_info = device.info()?;
        
        println!("Device {}: {} (S/N: {})", i, device_info.name(), device_info.serial_number());

        // Create a pipeline for the device - equivalent to std::make_shared<ob::Pipeline>(dev)
        let pipeline = Pipeline::new(&device)?;
        
        pipelines.push((i, pipeline));
    }

    // Shared storage for framesets - equivalent to std::map<int, std::shared_ptr<const ob::Frame>> framesets
    let framesets = Arc::new(Mutex::new(HashMap::<usize, FrameSet>::new()));

    // Start streams for all devices
    // Note: We don't store the context Arc long-term, following Orbbec's recommendations
    start_streams(&pipelines, framesets.clone())?;

    println!("All devices started. Press Ctrl+C to exit...");

    // Main loop - equivalent to while(win.run())
    loop {
        thread::sleep(Duration::from_millis(100));
        
        // Process latest frames from all devices
        {
            let framesets_lock = framesets.lock().unwrap();
            for (device_index, frameset) in framesets_lock.iter() {
                // In a real application, you would render or process the frames here
                println!(
                    "Device {}: Received frameset with timestamp {} µs",
                    device_index,
                    frameset.timestamp()
                );
            }
        }
    }
}

/// Start streams for all devices - equivalent to startStream() function
fn start_streams(
    pipelines: &[(usize, Pipeline)],
    framesets: Arc<Mutex<HashMap<usize, FrameSet>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    
    for (device_index, pipeline) in pipelines {
        let device_index = *device_index;
        
        // Create config to enable depth and color streams - equivalent to config->enableVideoStream()
        let config = Config::new()?;
        
        // Try to get and enable depth stream
        if let Ok(depth_profiles) = pipeline.get_stream_profiles(SensorType::Depth) {
            if let Ok(depth_profile) = depth_profiles.get_video_stream_profile(640, 480, orbbec_sdk::Format::Y16, 30) {
                config.enable_stream_with_profile(&depth_profile)?;
                println!("Device {}: Enabled depth stream", device_index);
            }
        }
        
        // Try to get and enable color stream
        if let Ok(color_profiles) = pipeline.get_stream_profiles(SensorType::Color) {
            if let Ok(color_profile) = color_profiles.get_video_stream_profile(640, 480, orbbec_sdk::Format::RGB, 30) {
                config.enable_stream_with_profile(&color_profile)?;
                println!("Device {}: Enabled color stream", device_index);
            }
        }

        // Clone the framesets Arc for the callback
        let framesets_callback = framesets.clone();
        
        // Start pipeline with callback - equivalent to pipe->start(config, callback)
        pipeline.start_with_callback(&config, move |frameset| {
            // This callback runs in a separate thread for each device
            // Equivalent to the lambda in C++: [deviceIndex](std::shared_ptr<ob::FrameSet> frameSet)
            let mut framesets_lock = framesets_callback.lock().unwrap();
            framesets_lock.insert(device_index, frameset);
        })?;
        
        println!("Device {}: Pipeline started with callback", device_index);
    }
    
    println!("All {} device pipelines started successfully!", pipelines.len());
    Ok(())
}

/// Stop all streams - equivalent to stopStream() function
fn _stop_streams(
    pipelines: &[(usize, Pipeline)],
    framesets: Arc<Mutex<HashMap<usize, FrameSet>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    
    for (device_index, pipeline) in pipelines {
        // Stop the pipeline - equivalent to pipe->stop()
        pipeline.stop()?;
        println!("Device {}: Pipeline stopped", device_index);
    }
    
    // Clear framesets - equivalent to framesets.clear()
    {
        let mut framesets_lock = framesets.lock().unwrap();
        framesets_lock.clear();
    }
    
    println!("All pipelines stopped and framesets cleared");
    Ok(())
}