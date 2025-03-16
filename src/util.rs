use std::i16;

use opus::Channels;

pub trait IntoChannels {
    fn into_channels(&self) -> Channels;
}

impl IntoChannels for u16 {
    /// Converts u16 into a Channels type.
    fn into_channels(&self) -> Channels {
        match self {
            1 => Channels::Mono,
            _ => Channels::Stereo,
        }
    }
}

pub trait FromChannels {
    fn from_channels(&self) -> usize;
}

impl FromChannels for Channels {
    fn from_channels(&self) -> usize {
        match self {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        }
    }
}

pub fn float_into_i16(input: &f32) -> i16 {
    (input * i16::MAX as f32) as i16
}

pub fn convert_to_stereo<T: Copy>(pcm_data: &[T]) -> Vec<T> {
    let input_length = pcm_data.len();
    let output_length = input_length * 2;

    let mut result = Vec::with_capacity(output_length);
    for sample in pcm_data {
        result.push(*sample);
        result.push(*sample);
    }
    result
}

pub fn convert_to_mono(data: &[f32]) -> Vec<f32> {
    data.chunks(2)
        .map(|stereo| (stereo[0] + stereo[1]) * 0.5) // Average L+R
        .collect()
}
