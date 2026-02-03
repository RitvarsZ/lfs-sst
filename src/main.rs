mod insim_io;
mod ui;
mod audio;

pub const DEBUG_AUDIO_RESAMPLING: bool = false;
pub const USE_GPU: bool = true;
pub const MODEL_PATH: &str = "models/small.en.bin";
pub const INSIM_HOST: &str = "127.0.0.1";
pub const INSIM_PORT: &str = "29999";
pub const MESSAGE_PREVIEW_TIMEOUT_SECS: u64 = 20;
pub const RECORDING_TIMEOUT_SECS: u8 = 10;
pub const MAX_MESSAGE_LEN: usize = 95;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut audio_pipeline, mut stt_rx) = audio::audio_pipeline::AudioPipeline::new().await?;
    let (insim, mut insim_rx, _insim_handle) = insim_io::init_insim().await?;
    let mut ui_context = crate::ui::UiContext::default();

    loop {
        ui_context.dispatch_ui_events(insim.clone()).await;

        tokio::select! {
            // Wait for message timeout to clear the preview.
            _ = ui_context.clear_message_timeout() => {}

            // Check if there are any messages from the STT thread.
            Some(msg) = stt_rx.recv() => {
                ui_context.handle_stt_message(msg);
            }

            // Check received insim events.
            Some(event) = insim_rx.recv() => {
                ui_context.handle_insim_event(
                    event,
                    insim.clone(),
                    &mut audio_pipeline
                ).await;
            }
        }
    }
}

