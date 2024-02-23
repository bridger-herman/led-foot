use toml::Table;
use std::fs;

const LED_CONFIG_PATH: &str = "led_config.toml";

/// Configuration for LEDs, which is loaded on system startup
#[derive(Debug)]
pub struct LedConfig {
    pub tty_name: String
}


impl LedConfig {
    /// Load from default config path on initialization
    pub fn new() -> LedConfig {
        let config_string = fs::read_to_string(LED_CONFIG_PATH).expect(&format!("Unable to load config file, {}", LED_CONFIG_PATH));
        let toml_config = config_string.parse::<Table>().expect(&format!("Unable to parse TOML config, {}", LED_CONFIG_PATH));
        let cfg = LedConfig {
            tty_name: toml_config.get("tty_name").map(|v| v.as_str().unwrap()).unwrap_or("/dev/ttyACM0").to_string()
        };

        debug!("Loaded LedConfig {:?}", cfg);

        cfg
    }
}