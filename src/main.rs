use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
                    .child(div().size_8().bg(gpui::yellow()))
                    .child(div().size_8().bg(gpui::black()))
                    .child(div().size_8().bg(gpui::white())),
            )
    }
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let audio_host = cpal::default_host();
        let audio_device = audio_host
            .default_output_device()
            .expect("No output device found.");
        let mut audio_config_range = audio_device
            .supported_output_configs()
            .expect("Error while querying audio configs.");
        let audio_config = audio_config_range
            .next()
            .expect("No supported config!?")
            .with_max_sample_rate();
        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let sample_format = audio_config.sample_format();
        let config = audio_config.into();
        let stream = match sample_format {
            SampleFormat::F32 => {
                audio_device.build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        // react to stream events and read or write stream data here.
                        for sample in data.iter_mut() {
                            *sample = 1.0.to_sample();
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                audio_device.build_output_stream(&config, write_silence::<i16>, err_fn, None)
            }
            SampleFormat::U16 => {
                audio_device.build_output_stream(&config, write_silence::<u16>, err_fn, None)
            }
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        }
        .unwrap();

        stream.play().unwrap();

        // Generating the window with gpui-rs
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap();
    });
}
