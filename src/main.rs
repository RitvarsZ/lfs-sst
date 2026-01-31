use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::{stt::maybe_dump_buffer_to_wav};
mod stt;
mod audio_input;

pub const DEBUG_AUDIO_RESAMPLING: bool = false;
pub const USE_GPU: bool = true;
pub const MODEL_PATH: &str = "models/small.en.bin";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Press SPACE to toggle recording. Press ESC to quit.");

    // 1 Setup speech-to-text
    let stt_ctx = stt::SttContext::init_stt();
    // 2 Setup audio capture and resampler.
    let mut audio_capture = audio_input::AudioStreamContext::init_audio_capture(&stt_ctx)?;
    // 3 Push-to-talk toggle loop
    let mut is_recording = false;

    loop {
        // 4 Read logs from STT thread
        while let Ok(msg) = stt_ctx.log_rx.try_recv() {
            println!("{}", msg);
        }

        // Poll keys
        if !event::poll(Duration::from_millis(20))? {
            continue;
        }


        if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => {
                    println!("👋 Exiting...");
                    break;
                },
                KeyCode::Char(' ') => {
                    if !is_recording {
                        println!("🎤 Recording...");
                        stt_ctx.audio_data.lock().unwrap().clear();
                        *stt_ctx.recording.lock().unwrap() = true;
                        is_recording = true;
                    } else {
                        println!("🛑 Sending for transcription...");
                        *stt_ctx.recording.lock().unwrap() = false;
                        let raw_samples = stt_ctx.audio_data.lock().unwrap().clone();
                        let outdata = audio_capture.resample_to_16k_mono(&raw_samples)?;
                        maybe_dump_buffer_to_wav(&outdata)?;
                        stt_ctx.audio_tx.send(outdata).unwrap();
                        is_recording = false;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

