use audioadapter_buffers::direct::InterleavedSlice;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::Resampler;
use whisper_rs::convert_stereo_to_mono_audio;

use crate::stt;

pub struct AudioStreamContext {
    pub input_channels: usize,
    pub sample_rate: usize,
    pub resampler: rubato::Fft<f32>,
    stream: cpal::Stream,
}

impl AudioStreamContext {
    fn new(input_channels: usize, sample_rate: usize, stream: cpal::Stream, resampler: rubato::Fft<f32>) -> Self {
        Self {
            input_channels,
            sample_rate,
            stream,
            resampler,
        }
    }

    pub fn init_audio_capture(stt_ctx: &stt::SttContext) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device");
        let input_config = device.default_input_config()?;
        let input_channels = input_config.channels() as usize;
        println!("Using input device: {}", device.description()?);
        if (input_channels != 1) && (input_channels != 2) {
            return Err(format!("Unsupported number of input channels: {}. Only mono and stereo are supported.", input_channels).into());
        }
        let sample_rate = input_config.sample_rate() as usize;
        let resampler = rubato::Fft::<f32>::new(
            sample_rate,
            16_000,
            1024,
            2,
            1,
            rubato::FixedSync::Both
        )?;

        let audio_data_clone = stt_ctx.audio_data.clone();
        let recording_clone = stt_ctx.recording.clone();

        let stream = device.build_input_stream(
            &input_config.into(),
            move |data: &[f32], _| {
                if *recording_clone.lock().unwrap() {
                    audio_data_clone.lock().unwrap().extend_from_slice(data);
                }
            },
            move |err| eprintln!("Audio error: {:?}", err),
            None,
        )?;

        stream.pause()?;
        Ok(Self::new(input_channels, sample_rate, stream, resampler))
    }

    // Todo: this assumes input comes from stream.
    // Can i keep an stt_ctx audio_data buffer clone here or smth?
    pub fn resample_to_16k_mono(&mut self, input: &[f32]) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mono = match self.input_channels {
            1 => input.to_vec(),
            2 => convert_stereo_to_mono_audio(input).expect("should be no half samples missing"),
            _ => return Err("Unsupported number of input channels".into()),
        };

        let nbr_input_frames = mono.len();
        let input_adapter = InterleavedSlice::new(&mono, 1, nbr_input_frames).unwrap();
        let mut outdata = vec![0.0; nbr_input_frames * 16_000 / self.sample_rate + 256];
        let nbr_out_frames = outdata.len();
        let mut output_adapter = InterleavedSlice::new_mut(&mut outdata, 1, nbr_out_frames).unwrap();

        let mut indexing = rubato::Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: None,
        };
        let mut input_frames_left = nbr_input_frames;
        let mut input_frames_next = self.resampler.input_frames_next();

        // Loop over all full chunks.
        // There will be some unprocessed input frames left after the last full chunk.
        // see the `process_f64` example for how to handle those
        // using `partial_len` of the indexing struct.
        // It is also possible to use the `process_all_into_buffer` method
        // to process the entire file (including any last partial chunk) with a single call.
        while input_frames_left >= input_frames_next {
            let (frames_read, frames_written) = self.resampler
                .process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
                .unwrap();

            indexing.input_offset += frames_read;
            indexing.output_offset += frames_written;
            input_frames_left -= frames_read;
            input_frames_next = self.resampler.input_frames_next();
        }

        Ok(outdata)
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

