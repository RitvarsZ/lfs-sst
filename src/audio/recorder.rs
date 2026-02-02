use cpal::{SampleRate, traits::{DeviceTrait, HostTrait}};
use tokio::{sync::mpsc::{self, Receiver}};

pub struct AudioInputConfig {
    pub input_channels: usize,
    pub sample_rate: SampleRate,
}

pub fn init() -> Result<
    (cpal::Stream, AudioInputConfig, Receiver<Vec<f32>>),
    Box<dyn std::error::Error>
> {
    let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>(10);

    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device");
    let input_config = match device.default_input_config() {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to get default input config: {}", e).into()),
    };
    let input_channels = input_config.channels() as usize;
    if (input_channels != 1) && (input_channels != 2) {
        return Err(format!("Unsupported number of input channels: {}. Only mono and stereo are supported.", input_channels).into());
    }

    let sample_rate = input_config.sample_rate();
    let stream = device.build_input_stream(
        &input_config.into(),
        move |data: &[f32], _| {
            match audio_tx.blocking_send(data.to_vec()) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to send audio data: {}", e),
            };
        },
        move |err| eprintln!("Audio error: {:?}", err),
        None,
    )?;

    println!("Using input device: {}", device.description()?);

    let config = AudioInputConfig {
        input_channels,
        sample_rate,
    };

    Ok((stream, config, audio_rx))
}
