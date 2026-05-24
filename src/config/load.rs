use super::{LayConfig, CONFIG_PATH};

impl LayConfig {
    pub fn load() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = format!("{home}/{CONFIG_PATH}");
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
                eprintln!("[lay] config parse error: {e}, using defaults");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}
