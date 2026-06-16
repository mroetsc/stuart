use std::path::PathBuf;

use crate::serial::{DataBits, FlowControl, NewlineEncoding, Parity, StopBits};

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct FileConfig {
    pub serial: SerialConfig,
    pub behavior: BehaviorConfig,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct SerialConfig {
    pub baud: Option<u32>,
    pub data_bits: Option<u8>,
    pub stop_bits: Option<u8>,
    pub parity: Option<String>,
    pub flow_control: Option<String>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct BehaviorConfig {
    pub local_echo: Option<bool>,
    pub outgoing_newline: Option<String>,
    pub keep_open: Option<bool>,
}

const VALID_PARITY: &[&str] = &["none", "even", "odd"];
const VALID_FLOW_CONTROL: &[&str] = &["none", "software", "hardware"];
const VALID_NEWLINE: &[&str] = &["cr", "lf", "crlf"];

pub fn load() -> Result<FileConfig, String> {
    let Some(path) = config_path() else {
        return Ok(FileConfig::default());
    };
    config::Config::builder()
        .add_source(config::File::from(path).required(false))
        .build()
        .and_then(|c| c.try_deserialize())
        .map_err(|error| error.to_string())
}

pub fn config_path_display() -> String {
    config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "(unknown)".to_string())
}

const DEFAULT_CONFIG: &str = include_str!("../example.config.toml");

pub fn write_default(force: bool) -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| "cannot determine config path".to_string())?;
    if path.exists() && !force {
        return Err(format!(
            "config file already exists at {}\nUse --force to overwrite.",
            path.display()
        ));
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|error| format!("failed to create config dir: {error}"))?;
    }
    std::fs::write(&path, DEFAULT_CONFIG)
        .map_err(|error| format!("failed to write config: {error}"))?;
    Ok(path)
}

pub fn validate(config: &FileConfig) -> Vec<String> {
    let mut errors = Vec::new();

    if let Some(baud) = config.serial.baud
        && baud == 0 {
            errors.push("serial.baud: must be greater than 0".to_string());
        }

    if let Some(data_bits) = config.serial.data_bits
        && !(5..=8).contains(&data_bits) {
            errors.push(format!(
                "serial.data_bits: got {data_bits}, valid values: 5 | 6 | 7 | 8"
            ));
        }

    if let Some(stop_bits) = config.serial.stop_bits
        && stop_bits != 1 && stop_bits != 2 {
            errors.push(format!(
                "serial.stop_bits: got {stop_bits}, valid values: 1 | 2"
            ));
        }

    check_str_field(
        &mut errors,
        "serial.parity",
        &config.serial.parity,
        VALID_PARITY,
    );
    check_str_field(
        &mut errors,
        "serial.flow_control",
        &config.serial.flow_control,
        VALID_FLOW_CONTROL,
    );
    check_str_field(
        &mut errors,
        "behavior.outgoing_newline",
        &config.behavior.outgoing_newline,
        VALID_NEWLINE,
    );

    errors
}

fn check_str_field(errors: &mut Vec<String>, field: &str, value: &Option<String>, valid: &[&str]) {
    let Some(value) = value else { return };
    if !valid.iter().any(|&v| v.eq_ignore_ascii_case(value)) {
        errors.push(format!(
            "{field}: got \"{value}\", valid values: {}",
            valid.join(" | ")
        ));
    }
}

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("stuart").join("config.toml"))
}

pub fn parse_data_bits(value: u8) -> DataBits {
    match value {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        _ => DataBits::Eight,
    }
}

pub fn parse_stop_bits(value: u8) -> StopBits {
    match value {
        2 => StopBits::Two,
        _ => StopBits::One,
    }
}

pub fn parse_parity(value: &str) -> Parity {
    match value.to_lowercase().as_str() {
        "even" => Parity::Even,
        "odd" => Parity::Odd,
        _ => Parity::None,
    }
}

pub fn parse_flow_control(value: &str) -> FlowControl {
    match value.to_lowercase().as_str() {
        "software" => FlowControl::Software,
        "hardware" => FlowControl::Hardware,
        _ => FlowControl::None,
    }
}

pub fn parse_newline(value: &str) -> NewlineEncoding {
    match value.to_lowercase().as_str() {
        "lf" => NewlineEncoding::LF,
        "crlf" => NewlineEncoding::CRLF,
        _ => NewlineEncoding::CR,
    }
}
