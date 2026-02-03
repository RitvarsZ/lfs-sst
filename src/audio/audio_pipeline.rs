use cpal::traits::StreamTrait;
use tokio::{sync::{mpsc::{self, Receiver}}, task::JoinHandle};

use crate::{RECORDING_TIMEOUT_SECS, audio::{self, speech_to_text::SttMessage}};

pub struct AudioPipeline {
    stream: cpal::Stream,
    is_recording_tx: tokio::sync::watch::Sender<bool>,
    resampler_handle: JoinHandle<()>,
    stt_handle: JoinHandle<()>,
    capture_handle: JoinHandle<()>,
}

impl AudioPipeline {
    pub async fn new() -> Result<(Self, Receiver<SttMessage>), Box<dyn std::error::Error>> {
        let (stream, stream_config, recorder_rx) = audio::recorder::init()?;
        let (resampled_rx, resampler_handle) = audio::resampler::init(
            recorder_rx,
            stream_config.sample_rate as usize,
            stream_config.input_channels,
        ).await?;
        let (stt_tx, audio_buffer_rx) = mpsc::channel::<Vec<f32>>(1);
        let (is_recording_tx, is_recording_rx) = tokio::sync::watch::channel(false);
        let capture_handle = init_audio_capture(
            resampled_rx,
            stt_tx,
            is_recording_rx
        ).await?;
        let (stt_rx, stt_handle) = audio::speech_to_text::init(audio_buffer_rx).await?;

        let pipeline = AudioPipeline {
            is_recording_tx,
            stream,
            resampler_handle,
            stt_handle,
            capture_handle,
        };

        Ok((pipeline, stt_rx))
    }

    fn pause(&self) {
        match self.stream.pause() {
            Ok(()) => (),
            Err(e) => eprintln!("Failed to pause audio stream: {}", e),
        };
    }

    fn play(&self) {
        match self.stream.play() {
            Ok(()) => (),
            Err(e) => eprintln!("Failed to start audio stream: {}", e),
        }
    }

    /// Start stream and accumulate resampled audio into buffer.
    /// If buffer reaches timeout size, stop recording and transcribe.
    pub async fn start_recording(&self) {
        self.play();
        self.is_recording_tx.send(true).expect("Failed to send recording state");
    }

    /// Stop stream, send accumulated audio_buffer to STT, and clear buffer.
    pub async fn stop_recording_and_transcribe(&self) {
        self.pause();
        self.is_recording_tx.send(false).expect("Failed to send recording state");
    }
}

async fn init_audio_capture(
    mut rx: mpsc::Receiver<Vec<f32>>,
    tx: mpsc::Sender<Vec<f32>>,
    mut is_recording_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    let handle = tokio::spawn(async move {
        let mut buffer = Vec::<f32>::with_capacity(16_000 * RECORDING_TIMEOUT_SECS as usize);

        loop {
            tokio::select! {
                _ = is_recording_rx.changed() => {
                    if !*is_recording_rx.borrow() && !buffer.is_empty() {
                        if tx.send(buffer.clone()).await.is_err() {
                            break;
                        }
                        buffer.clear();
                    }
                }

                Some(data) = rx.recv() => {
                    if *is_recording_rx.borrow() {
                        buffer.extend_from_slice(&data);
                        // todo: figure out how to stop recording.
                        // if buffer.len() >= 16_000 * RECORDING_TIMEOUT_SECS as usize {
                        //     if tx.send(buffer.clone()).await.is_err() {
                        //         break;
                        //     }
                        //     buffer.clear();
                        // }
                    }
                }
            }
        }
    });

    Ok(handle)
}

