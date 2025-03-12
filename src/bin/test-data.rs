use std::sync::{Arc, Mutex};

use cpal::{
    InputCallbackInfo, OutputCallbackInfo, default_host,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use opus::{Channels, Decoder, Encoder};
use test_gpui::util::{convert_to_mono, convert_to_stereo};

const ENCODING_SAMPLE_RATE: u32 = 48_000;
// 48_000 * channels * 20ms / 1000;
const FRAME_SIZE: usize = 960;

fn main() {
    // Prepping audio input
    let host = default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();
    println!("Input Sample Rate: {:?}", &config.sample_rate());
    println!("Input Channel Count: {:?}", &config.channels());

    let mut encoded_bytes: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let mut encoded_input_clone = encoded_bytes.clone();
    let mut encoded_output_clone = encoded_bytes.clone();

    // Initialize the encoder
    let mut encoder = Encoder::new(
        ENCODING_SAMPLE_RATE,
        Channels::Stereo,
        opus::Application::Voip,
    )
    .unwrap();

    let mut buffer = Vec::with_capacity(FRAME_SIZE);

    let data_callback = move |data: &[f32], _: &InputCallbackInfo| {
        buffer.extend_from_slice(&convert_to_stereo(data));
        if buffer.len() >= FRAME_SIZE * 2 {
            let drain = buffer.drain(..FRAME_SIZE * 2).collect::<Vec<f32>>();
            let mut encoded_buffer = [0u8; FRAME_SIZE * 2];
            let size = encoder
                .encode_float(&drain[..], &mut encoded_buffer)
                .unwrap();
            let mut encoded = encoded_input_clone.lock().unwrap();
            encoded.extend_from_slice(&encoded_buffer[..size]);
        }
    };

    let error_callback = move |error| eprintln!("Error: {}", error);

    let stream = device
        .build_input_stream(&config.into(), data_callback, error_callback, None)
        .unwrap();

    stream.play().unwrap();

    let output_device = host.default_output_device().unwrap();
    let mut output_config = output_device.default_output_config().unwrap();
    let output_channels = output_config.channels();
    println!("Output sample rate: {:?}", &output_config.sample_rate());
    println!("Output Channel Count: {:?}", &output_config.channels());

    let mut decoder = Decoder::new(ENCODING_SAMPLE_RATE, Channels::Stereo).unwrap();

    let mut output_buffer_mono = [0f32; FRAME_SIZE * 2];
    let mut overflow_buffer = Vec::<f32>::new();

    let output_data_callback = move |data: &mut [f32], _: &OutputCallbackInfo| {
        dbg!(data.len());
        let mut encoded = encoded_output_clone.lock().unwrap();
        if encoded.is_empty() {
            return;
        }
        if encoded.len() >= 0 {
            let size = decoder
                .decode_float(&encoded, &mut output_buffer_mono, false)
                .unwrap();

            dbg!(size);
            if output_channels >= 2 {
                overflow_buffer.extend_from_slice(&output_buffer_mono[..size]);
            } else {
                overflow_buffer.extend_from_slice(&convert_to_mono(&output_buffer_mono[..size]));
            }
        }
        let to_consume = data.len().min(overflow_buffer.len());
        data[..to_consume].copy_from_slice(&overflow_buffer[..to_consume]);
        overflow_buffer.drain(..to_consume);
        dbg!(overflow_buffer.len());
        encoded.drain(..);
    };

    let output_stream = output_device
        .build_output_stream(
            &output_config.into(),
            output_data_callback,
            error_callback,
            None,
        )
        .unwrap();

    output_stream.play().unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
