use std::{sync::mpsc::{Receiver, Sender}, thread};

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Resampler};
use whisper_rs::convert_stereo_to_mono_audio;

/**
* receive audio from mic on channel.
* resample to 16k mono.
* send to stt for transcription.
*/
pub fn init_resampler(
    audio_in: Receiver<Vec<f32>>,
    audio_out: Sender<Vec<f32>>,
    sample_rate: usize,
    input_channels: usize,
) -> Receiver<Vec<f32>> {
    let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();

    thread::spawn(move || {
        let mut resampler = match rubato::Fft::<f32>::new(sample_rate, 16_000, 1024, 2, 1, rubato::FixedSync::Both) {
            Ok(r) => r,
            Err(e) => panic!("Failed to create resampler: {}", e),
        };

        while let Ok(samples) = audio_in.recv() {
            let mono = match input_channels {
                1 => samples,
                2 => convert_stereo_to_mono_audio(&samples).expect("should be no half samples missing"),
                _ => panic!("Unsupported number of input channels: {}", input_channels),
            };

            let nbr_input_frames = mono.len();
            let input_adapter = InterleavedSlice::new(&mono, 1, nbr_input_frames).unwrap();
            let mut outdata = vec![0.0; nbr_input_frames * 16_000 / sample_rate + 256];
            let nbr_out_frames = outdata.len();
            let mut output_adapter = InterleavedSlice::new_mut(&mut outdata, 1, nbr_out_frames).unwrap();

            let mut indexing = rubato::Indexing {
                input_offset: 0,
                output_offset: 0,
                active_channels_mask: None,
                partial_len: None,
            };
            let mut input_frames_left = nbr_input_frames;
            let mut input_frames_next = resampler.input_frames_next();

            // Loop over all full chunks.
            // There will be some unprocessed input frames left after the last full chunk.
            // see the `process_f64` example for how to handle those
            // using `partial_len` of the indexing struct.
            // It is also possible to use the `process_all_into_buffer` method
            // to process the entire file (including any last partial chunk) with a single call.
            while input_frames_left >= input_frames_next {
                let (frames_read, frames_written) = resampler
                    .process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
                    .unwrap();

                indexing.input_offset += frames_read;
                indexing.output_offset += frames_written;
                input_frames_left -= frames_read;
                input_frames_next = resampler.input_frames_next();
            }

            let _ = tx.send(outdata.clone());

            audio_out.send(outdata).expect("Failed to send resampled audio");
        }
    });

    rx
}

