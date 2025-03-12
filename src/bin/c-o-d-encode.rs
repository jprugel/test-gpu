use cpal::{
    BufferSize, SampleFormat, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use opus::{Channels, Decoder, Encoder};
use std::sync::{Arc, Mutex};

const ENCODING_SAMPLE_RATE: u32 = 48_000;
const ENCODING_CHANNELS: Channels = Channels::Stereo;
const ENCODING_MS: usize = (ENCODING_SAMPLE_RATE as usize) * 2 * 20 / 1000;

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
        buffer_size: BufferSize::Fixed(960),
    };

    // Shared buffer between input and output
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

    // Clone the buffer for input handling
    let input_buffer = Arc::clone(&audio_buffer);

    let mut encoder = Encoder::new(
        ENCODING_SAMPLE_RATE,
        ENCODING_CHANNELS,
        opus::Application::Voip,
    )
    .unwrap();
    let mut decoder = Decoder::new(ENCODING_SAMPLE_RATE, ENCODING_CHANNELS).unwrap();
    let encoded_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let input_encoded_buffer = Arc::clone(&encoded_buffer);
    let output_encoded_buffer = Arc::clone(&encoded_buffer);

    // Start input stream (recording)
    let input_stream = input_device
        .build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                let mut buffer = input_buffer.lock().unwrap();
                buffer.extend_from_slice(data);
                println!("Captured {} samples", data.len());
                if buffer.len() >= ENCODING_MS {
                    let mut abuffer = input_encoded_buffer.lock().unwrap();
                    let mut tbuffer = [0u8; ENCODING_MS];
                    let size = encoder
                        .encode_float(&buffer[..ENCODING_MS], &mut tbuffer)
                        .expect("Failed to encode buffer");
                    abuffer.extend_from_slice(&tbuffer[..size]);
                }
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
                let mut buffer = output_encoded_buffer.lock().unwrap();
                let mut decoded_buffer = [0f32; ENCODING_MS];
                if buffer.len() >= 1 {
                    let size = decoder
                        .decode_float(&buffer, &mut decoded_buffer, true)
                        .expect("Failed to decode buffer");
                    let needed_samples = output.len();
                    // Fill output with available samples
                    for (out_sample, in_sample) in output.iter_mut().zip(decoded_buffer.iter()) {
                        *out_sample = *in_sample;
                    }

                    // Drain the processed samples
                    buffer.drain(..needed_samples.min(size));
                }
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
