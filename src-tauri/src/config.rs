use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug)]
pub struct Addresses {
    pub clear_core: String,
    pub addr: [u8; 4],
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MotorConfig {
    pub id: usize,
    pub scale: usize,
    pub acceleration: f64,
    pub min_speed: f64,
    pub max_speed: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhotoEyeConfig {
    #[serde(with = "duration_serde")]
    pub sample_period: Duration,
    pub sample_number: usize,
    pub input_id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HatchConfig {
    pub motor_id: usize,
    pub open_input: usize,
    pub close_input: usize,
    pub velocity: f64,
    pub acceleration: f64,
    pub scale: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhidgetConfig {
    pub sn: i32,
    pub coefficients: [f64; 4],
    #[serde(with = "duration_serde")]
    pub data_interval: Duration,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DispenseConfig {
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SetpointConfig {
    pub empty: f64,
    pub filling_threshold: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Pins {
    pub sudo: usize,
    pub manager: usize,
    pub operator: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub phidget: PhidgetConfig,
    pub hatch: HatchConfig,
    pub photo_eye: PhotoEyeConfig,
    pub motor: MotorConfig,
    pub addresses: Addresses,
    pub dispense: DispenseConfig,
    pub setpoint: SetpointConfig,
    pub pins: Pins,
}

impl Config {
    pub fn load() -> Self {
        const PATH: &str = ".config/ichibu/controls_config.toml";
        let home_dir = env::var_os("HOME")
            .expect("Fatal, no home directory found")
            .into_string()
            .unwrap();

        let path = format!("{}/{}", &*home_dir, PATH);

        let config_text = fs::read_to_string(path).unwrap();
        let config: Config = toml::from_str(&config_text).expect("No config file loaded");
        config
    }
}

mod duration_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}
