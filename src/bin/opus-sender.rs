use std::net::UdpSocket;

use cpal::{
    InputCallbackInfo, SampleRate, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use opus::{Application, Channels, Encoder};
use test_gpui::util::{convert_to_stereo, float_into_i16};

const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: Channels = Channels::Mono;
const STERIO20MS: usize = SAMPLE_RATE as usize * 20 / 1000;

fn main() {
    let host = default_host();
    let device = host
        .default_input_device()
        .expect("Failed to get default input device.");
    let config = device
        .default_input_config()
        .expect("Failed to get default input config.");

    let mut encoder = Encoder::new(SAMPLE_RATE, CHANNELS, Application::Voip)
        .expect("Failed to initialize encoder.");
    let mut buffer = Vec::new();
    let input_channels = config.channels();

    let socket = UdpSocket::bind("0.0.0.0:5000").expect("Failed to bind UDP socket");

    let stream = device
        .build_input_stream(
            &config.into(),
            move |pcm_data: &[f32], _: &InputCallbackInfo| {
                buffer.extend_from_slice(pcm_data);

                if buffer.len() >= STERIO20MS {
                    println!("Size of buffer before encoding: {}", buffer.len());

                    // Convert mono to stereo
                    let stereo_buffer = convert_to_stereo(&buffer);

                    // Convert stereo float data to i16
                    let buffer_i16: Vec<i16> = stereo_buffer.iter().map(float_into_i16).collect();

                    // Prepare buffer for encoding
                    let mut encoded_audio = [0u8; STERIO20MS];
                    let size = encoder
                        .encode(&buffer_i16[..STERIO20MS], &mut encoded_audio)
                        .expect("Failed to encode audio.");
                    println!("Size of encoded data: {}", size);

                    // Send packet over UDP
                    let packet = socket
                        .send_to(&encoded_audio[..size], "0.0.0.0:5001")
                        .expect("Failed to send packet!.");
                    println!("Sent a packet of size: {}", packet);

                    // Remove processed data from buffer
                    buffer.drain(..STERIO20MS);
                    println!("Size of buffer after draining: {}", buffer.len());
                }
            },
            move |err| eprintln!("error: {}", err),
            None,
        )
        .expect("Failed to create stream.");

    stream.play().expect("Failed to play stream");

    // Keep running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
