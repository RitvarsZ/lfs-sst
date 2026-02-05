use std::fmt::Display;
use serde::Deserialize;
use tracing::level_filters::LevelFilter;

pub const CONFIG_PATH: &str = "config.toml";

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    ValidationError(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO Error - {}", e),
            ConfigError::Parse(e) => write!(f, "Parse Error - {}", e),
            ConfigError::ValidationError(e) => write!(f, "Validation Error - {}", e),
        }
    }
}


#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> LevelFilter {
        match level {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn  => LevelFilter::WARN,
            LogLevel::Info  => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub btn_id_offset: u8,
    pub debug_log_level: LogLevel,
    pub chat_channels: Vec<ChatChannel>,
    pub debug_audio_resampling: bool,
    pub insim_host: String,
    pub insim_port: String,
    pub message_preview_timeout_secs: u64,
    pub model_path: String,
    pub recording_timeout_secs: u8,
    pub ui_offset_left: u8,
    pub ui_offset_top: u8,
    pub ui_scale: u8,
    pub use_gpu: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ChatChannel {
    pub display: String,
    pub prefix: String,
}

impl PartialEq for ChatChannel {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix
    }
}


impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Config {{ insim_host: {}, insim_port: {}, chat_channels: {:?}, model_path: {}, message_preview_timeout_secs: {}, recording_timeout_secs: {}, ui_scale: {}, ui_offset_top: {}, ui_offset_left: {}, btn_id_offset: {}, debug_audio_resampling: {}, use_gpu: {} }}",
            self.insim_host, self.insim_port, self.chat_channels, self.model_path, self.message_preview_timeout_secs, self.recording_timeout_secs, self.ui_scale, self.ui_offset_top, self.ui_offset_left, self.btn_id_offset, self.debug_audio_resampling, self.use_gpu)
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(CONFIG_PATH)
            .map_err(ConfigError::Io)?;
        let config: Self = toml::from_str(&contents)
            .map_err(ConfigError::Parse)?;
        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.chat_channels.is_empty() {
            return Err(ConfigError::ValidationError("Result<(), String>".into()));
        }

        if self.ui_scale == 0 {
            return Err(ConfigError::ValidationError("UI scale must be greater than 0.".into()))
        }

        if self.ui_offset_top > 200 {
            return Err(ConfigError::ValidationError("UI offset top must be between 0 and 200.".into()))
        }

        if self.ui_offset_left > 200 {
            return Err(ConfigError::ValidationError("UI offset left must be between 0 and 200.".into()))
        }
        if self.model_path.is_empty() {
            return Err(ConfigError::ValidationError("Model path cannot be empty.".into()))
        }
        if self.btn_id_offset > 230 {
            return Err(ConfigError::ValidationError("Button ID offset must be between 0 and 230.".into()))
        }

        for channel in &self.chat_channels {
            if channel.display.is_empty() {
                return Err(ConfigError::ValidationError("Chat channel display name cannot be empty.".into()))
            }
        }

        Ok(())
    }
}
