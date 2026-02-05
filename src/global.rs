use crate::config::Config;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
    let cfg = match Config::load().map_err(|e| {
        eprintln!("Failed to load config: {}", e);
        e
    }) {
        Ok(cfg) => cfg,
        Err(_) => { panic!(); }
    };
    cfg.validate().expect("Invalid config.toml");
    Arc::new(cfg)
});

