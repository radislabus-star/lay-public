use super::{config_path, LayConfig};

impl LayConfig {
    pub fn load() -> Self {
        match std::fs::read_to_string(config_path()) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
                eprintln!("[lay] config parse error: {e}, using defaults");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}
