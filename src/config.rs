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

pub fn load() -> FileConfig {
    let Some(path) = config_path() else {
        return FileConfig::default();
    };
    config::Config::builder()
        .add_source(config::File::from(path).required(false))
        .build()
        .and_then(|c| c.try_deserialize())
        .unwrap_or_default()
}

pub fn config_path_display() -> String {
    config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unknown)".to_string())
}

const DEFAULT_CONFIG: &str = r#"# stuart configuration
# All values are optional. Unset fields fall back to the app default.
# CLI flags always override these.

[serial]
baud = 115200
data_bits = 8       # 5 | 6 | 7 | 8
stop_bits = 1       # 1 | 2
parity = "none"     # none | even | odd
flow_control = "none"  # none | software | hardware

[behavior]
local_echo = false
outgoing_newline = "cr"  # cr | lf | crlf
keep_open = true
"#;

pub fn write_default(force: bool) -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| "cannot determine config path".to_string())?;
    if path.exists() && !force {
        return Err(format!(
            "config file already exists at {}\nUse --force to overwrite.",
            path.display()
        ));
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("failed to create config dir: {e}"))?;
    }
    std::fs::write(&path, DEFAULT_CONFIG).map_err(|e| format!("failed to write config: {e}"))?;
    Ok(path)
}

pub fn validate(cfg: &FileConfig) -> Vec<String> {
    let mut errors = Vec::new();

    if let Some(baud) = cfg.serial.baud
        && baud == 0 {
            errors.push("serial.baud: must be greater than 0".to_string());
        }

    if let Some(db) = cfg.serial.data_bits
        && !(5..=8).contains(&db) {
            errors.push(format!(
                "serial.data_bits: got {db}, valid values: 5 | 6 | 7 | 8"
            ));
        }

    if let Some(sb) = cfg.serial.stop_bits
        && sb != 1 && sb != 2 {
            errors.push(format!("serial.stop_bits: got {sb}, valid values: 1 | 2"));
        }

    if let Some(ref p) = cfg.serial.parity {
        let valid = ["none", "even", "odd"];
        if !valid.contains(&p.to_lowercase().as_str()) {
            errors.push(format!(
                "serial.parity: got \"{p}\", valid values: none | even | odd"
            ));
        }
    }

    if let Some(ref fc) = cfg.serial.flow_control {
        let valid = ["none", "software", "hardware", "xon/xoff", "rts/cts"];
        if !valid.contains(&fc.to_lowercase().as_str()) {
            errors.push(format!(
                "serial.flow_control: got \"{fc}\", valid values: none | software | hardware"
            ));
        }
    }

    if let Some(ref nl) = cfg.behavior.outgoing_newline {
        let valid = ["cr", "lf", "crlf", "cr+lf"];
        if !valid.contains(&nl.to_lowercase().as_str()) {
            errors.push(format!(
                "behavior.outgoing_newline: got \"{nl}\", valid values: cr | lf | crlf"
            ));
        }
    }

    errors
}

fn config_path() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".config")
            });
        return Some(base.join("stuart").join("config.toml"));
    }
    #[cfg(windows)]
    {
        return Some(
            PathBuf::from(std::env::var("APPDATA").ok()?)
                .join("stuart")
                .join("config.toml"),
        );
    }
    #[allow(unreachable_code)]
    None
}

pub fn parse_data_bits(v: u8) -> DataBits {
    match v {
        5 => DataBits::Five,
        6 => DataBits::Six,
        7 => DataBits::Seven,
        _ => DataBits::Eight,
    }
}

pub fn parse_stop_bits(v: u8) -> StopBits {
    match v {
        2 => StopBits::Two,
        _ => StopBits::One,
    }
}

pub fn parse_parity(v: &str) -> Parity {
    match v.to_lowercase().as_str() {
        "even" => Parity::Even,
        "odd" => Parity::Odd,
        _ => Parity::None,
    }
}

pub fn parse_flow_control(v: &str) -> FlowControl {
    match v.to_lowercase().as_str() {
        "software" | "xon/xoff" => FlowControl::Software,
        "hardware" | "rts/cts" => FlowControl::Hardware,
        _ => FlowControl::None,
    }
}

pub fn parse_newline(v: &str) -> NewlineEncoding {
    match v.to_lowercase().as_str() {
        "lf" => NewlineEncoding::LF,
        "crlf" | "cr+lf" => NewlineEncoding::CRLF,
        _ => NewlineEncoding::CR,
    }
}
