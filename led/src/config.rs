use rpi_led_panel::{HardwareMapping, RGBMatrixConfig};
use serde::Deserialize;

pub struct GameConfig {
}

#[derive(Debug)]
pub struct ConfigParser {
    parsed_config: ParsedConfig,
}

impl ConfigParser {
    pub fn from_file(filename: String) -> Result<Self, String> {
        let contents = std::fs::read_to_string(filename).map_err(|e| e.to_string())?;
        let parsed: ParsedConfig = toml::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(ConfigParser {
            parsed_config: parsed,
        })
    }

    pub fn get_game_config(&self) -> GameConfig {
        let config: GameConfig = GameConfig {
        };

        config
    }

    pub fn get_led_config(&self) -> Result<RGBMatrixConfig, String> {
        use std::str::FromStr;
        let mut config = rpi_led_panel::RGBMatrixConfig::default();
        let parsed_led_config = &self
            .parsed_config
            .led_config
            .as_ref()
            .ok_or("Missing LED config")?;

        config.hardware_mapping = HardwareMapping::regular();
        config.rows = parsed_led_config.rows;
        config.cols = parsed_led_config.cols;
        if let Some(v) = parsed_led_config.chain_length {
            config.chain_length = v;
        }
        if let Some(v) = parsed_led_config.num_chains {
            config.parallel = v;
        }
        if let Some(v) = parsed_led_config.pwm_bits {
            config.pwm_bits = v;
        }
        if let Some(v) = parsed_led_config.pwm_lsb_ns {
            config.pwm_lsb_nanoseconds = v;
        }
        if let Some(v) = parsed_led_config.pwm_dither_bits {
            config.dither_bits = v;
        }
        if let Some(v) = parsed_led_config.gpio_slowdown {
            config.slowdown = Some(v);
        }
        if let Some(arr) = parsed_led_config.pixel_mappers.as_ref() {
            for s in arr {
                let mapper =
                    rpi_led_panel::NamedPixelMapperType::from_str(s).map_err(|e| e.to_string());
                config.pixelmapper.push(mapper?);
            }
        }
        if let Some(v) = parsed_led_config.brightness {
            config.led_brightness = v;
        }

        // Some information comes from the main game config section
        config.refresh_rate = 120;

        Ok(config)
    }
}


////////////////////////////////////////////

#[derive(Debug, Clone, Deserialize)]
struct ParsedConfig {
    led_config: Option<ParsedLedConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct ParsedLedConfig {
    rows: usize,
    cols: usize,
    chain_length: Option<usize>,
    num_chains: Option<usize>,
    pwm_bits: Option<usize>,
    pwm_dither_bits: Option<usize>,
    pwm_lsb_ns: Option<u32>,
    gpio_slowdown: Option<u32>,
    pixel_mappers: Option<Vec<String>>,
    brightness: Option<u8>,
}
