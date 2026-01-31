use std::sync::mpsc::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioStreamContext {
    pub input_channels: usize,
    pub sample_rate: usize,
    stream: cpal::Stream,
}

impl AudioStreamContext {
    fn new(input_channels: usize, sample_rate: usize, stream: cpal::Stream) -> Self {
        Self {
            input_channels,
            sample_rate,
            stream,
        }
    }

    pub fn init_audio_capture(audio_out: Sender<Vec<f32>>) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device");
        let input_config = device.default_input_config()?;
        let input_channels = input_config.channels() as usize;
        println!("Using input device: {}", device.description()?);
        if (input_channels != 1) && (input_channels != 2) {
            return Err(format!("Unsupported number of input channels: {}. Only mono and stereo are supported.", input_channels).into());
        }
        let sample_rate = input_config.sample_rate() as usize;

        let stream = device.build_input_stream(
            &input_config.into(),
            move |data: &[f32], _| {
                audio_out.send(data.to_vec()).expect("Failed to send audio data");
            },
            move |err| eprintln!("Audio error: {:?}", err),
            None,
        )?;

        stream.pause()?;
        Ok(Self::new(input_channels, sample_rate, stream))
    }


    pub fn pause_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream.pause() {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to pause audio stream: {}", e).into()),
        }
    }

    pub fn start_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.stream.play() {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to start audio stream: {}", e).into()),
        }
    }
}

