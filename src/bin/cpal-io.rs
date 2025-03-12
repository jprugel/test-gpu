use cpal::{
    BufferSize, SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, Mutex};

fn main() {
    let host = cpal::default_host();

    // Select input and output devices
    let input_device = host
        .default_input_device()
        .expect("No default input device.");
    let output_device = host
        .default_output_device()
        .expect("No default output device.");

    println!("Input device: {}", input_device.name().unwrap());
    println!("Output device: {}", output_device.name().unwrap());

    // Get input and output configurations
    let input_config = input_device
        .default_input_config()
        .expect("Failed to get input config.");
    let output_config = output_device
        .default_output_config()
        .expect("Failed to get output config.");

    println!("Input config: {:?}", input_config);
    println!("Output config: {:?}", output_config);

    // Ensure both input and output use the same sample rate
    let sample_rate = input_config.sample_rate().0;
    let channels = input_config.channels();

    let stream_config = StreamConfig {
        channels,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: BufferSize::Fixed(1024),
    };

    // Shared buffer between input and output
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

    // Clone the buffer for input handling
    let input_buffer = Arc::clone(&audio_buffer);

    // Start input stream (recording)
    let input_stream = input_device
        .build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                let mut buffer = input_buffer.lock().unwrap();
                buffer.extend_from_slice(data);
                println!("Captured {} samples", data.len());
            },
            |err| eprintln!("Input stream error: {}", err),
            None,
        )
        .expect("Failed to create input stream");

    // Clone buffer for output handling
    let output_buffer = Arc::clone(&audio_buffer);

    // Start output stream (playback)
    let output_stream = output_device
        .build_output_stream(
            &stream_config,
            move |output: &mut [f32], _| {
                let mut buffer = output_buffer.lock().unwrap();

                let available_samples = buffer.len();
                let needed_samples = output.len();

                if available_samples < needed_samples {
                    println!(
                        "Buffer underflow! Available: {}, Needed: {}",
                        available_samples, needed_samples
                    );
                }

                // Fill output with available samples
                for (out_sample, in_sample) in output.iter_mut().zip(buffer.iter()) {
                    *out_sample = *in_sample;
                }

                // Drain the processed samples
                buffer.drain(..needed_samples.min(available_samples));
            },
            |err| eprintln!("Output stream error: {}", err),
            None,
        )
        .expect("Failed to create output stream");

    // Start both streams
    input_stream.play().expect("Failed to start input stream");
    output_stream.play().expect("Failed to start output stream");

    println!("Listening... Press Ctrl+C to stop.");

    // Keep running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
