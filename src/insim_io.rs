use crate::{INSIM_HOST, INSIM_PORT};

#[allow(dead_code)]
pub enum InsimEvent {
    ToggleRecording,
    AcceptMessage,
    NextChannel,
    PeviousChannel,
    IsInGame(bool),
}

impl InsimEvent {
    pub fn from_string(cmd: String) -> Option<InsimEvent> {
        match cmd.as_str() {
            "stt talk" => Some(InsimEvent::ToggleRecording),
            "stt accept" => Some(InsimEvent::AcceptMessage),
            _ => None,
        }
    }
}

pub fn init_message_io(
    event_tx: tokio::sync::mpsc::Sender<InsimEvent>,
    mut command_rx: tokio::sync::mpsc::Receiver<insim::Packet>,
) {
    tokio::spawn(async move {
        let mut conn = match insim::tcp(format!("{}:{}", INSIM_HOST, INSIM_PORT))
            .isi_iname("lfs-stt".to_owned())
            .isi_flag_local(true)
            .connect_async().await {
            Ok(c) => c,
            Err(err) => {
                println!("Failed to connect to INSIM: {}", err);
                return;
            },
        };

        // Request initial game state info.
        let _ = conn.write(insim::Packet::Tiny(insim::insim::Tiny{
            subt: insim::insim::TinyType::Sst,
            reqi: insim::identifiers::RequestId::from(1),
        })).await;

        loop {
            tokio::select! {
                Some(packet) = command_rx.recv() => {
                    let _ = conn.write(packet).await;
                }

                packet = conn.read() => {
                    match packet {
                        Ok(packet) => {
                            match packet {
                                insim::Packet::Mso(mso) => {
                                    if let Some(cmd) = InsimEvent::from_string(mso.msg) {
                                        let _ = event_tx.send(cmd).await;
                                    }
                                },
                                insim::Packet::Sta(sta) => {
                                    let _ = event_tx.send(InsimEvent::IsInGame(sta.flags.is_in_game())).await;
                                }
                                _ => {}
                            }
                        },
                        Err(e) => {
                            // Insim probably disconnected which means game closed.
                            // If not, we are cooked here.
                            panic!("Error reading from INSIM: {}", e);
                        },

                    }
                }
            }
        }
    });
}
