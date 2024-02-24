use toml::Table;
use std::fs;

const LED_CONFIG_PATH: &str = "led_config.toml";

/// Configuration for LEDs, which is loaded on system startup
// TODO: Use config crate
#[derive(Debug)]
pub struct LedConfig {
    pub tty_name: String,
    pub sequence_resolution: f32,
}


impl LedConfig {
    /// Load from default config path on initialization
    pub fn new() -> LedConfig {
        let config_string = fs::read_to_string(LED_CONFIG_PATH).expect(&format!("Unable to load config file, {}", LED_CONFIG_PATH));
        let toml_config = config_string.parse::<Table>().expect(&format!("Unable to parse TOML config, {}", LED_CONFIG_PATH));
        let cfg = LedConfig {
            tty_name: toml_config.get("tty_name").and_then(|v| v.as_str()).unwrap_or("/dev/ttyACM0").to_string(),
            sequence_resolution: toml_config.get("sequence_resolution").and_then(|v| v.as_float()).unwrap_or(30.0) as f32,
        };

        debug!("Loaded LedConfig {:?}", cfg);

        cfg
    }
}