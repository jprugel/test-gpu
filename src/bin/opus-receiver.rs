use cpal::{
    BufferSize, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use opus::{Channels, Decoder};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

fn convert_mono_to_stereo(pcm_data: &[f32]) -> Vec<f32> {
    let mut stereo_data = Vec::with_capacity(pcm_data.len() * 2);
    for &sample in pcm_data.iter() {
        stereo_data.push(sample); // Left channel
        stereo_data.push(sample); // Right channel
    }
    stereo_data
}

fn main() {
    let host = default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");

    let supported_config = device
        .default_output_config()
        .expect("Failed to get supported config");

    let mut config: cpal::StreamConfig = supported_config.clone().into();
    config.buffer_size = BufferSize::Fixed(4096);

    let sample_rate = config.sample_rate.0;
    let channels = config.channels;

    println!(
        "Using sample rate: {} Hz, channels: {}",
        sample_rate, channels
    );

    let mut decoder =
        Decoder::new(sample_rate, Channels::Stereo).expect("Failed to create Opus decoder");

    let socket = UdpSocket::bind("0.0.0.0:5001").expect("Failed to bind UDP socket");
    let audio_buffer = Arc::new(Mutex::new(Vec::new()));

    let buffer_clone = Arc::clone(&audio_buffer);
    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buffer = buffer_clone.lock().unwrap();
                println!("Playing {} samples", buffer.len());
                for (sample, value) in output.iter_mut().zip(buffer.drain(..)) {
                    *sample = value; // Play data or silence if buffer is empty
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .expect("Failed to create output stream");

    stream.play().expect("Failed to start output stream");

    // UDP receiving and decoding thread
    let buffer_clone = Arc::clone(&audio_buffer);
    thread::spawn(move || {
        let mut packet = [0; 960];
        loop {
            if let Ok((size, src)) = socket.recv_from(&mut packet) {
                println!(
                    "Received {} bytes from {} with data {:?}",
                    size,
                    src,
                    packet.len()
                );
                let mut pcm_data = [0_i16; 960]; // Buffer for decoded PCM
                match decoder.decode(&packet[..size], &mut pcm_data, true) {
                    Ok(len) => {
                        println!("Decoded {} samples", len);
                        let mut buffer = buffer_clone.lock().unwrap();
                        buffer.extend(pcm_data[..len].iter().map(|&x| x as f32 / i16::MAX as f32)); // Convert i16 -> f32
                    }
                    Err(e) => eprintln!("Error decoding: {}", e),
                }
            }
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
