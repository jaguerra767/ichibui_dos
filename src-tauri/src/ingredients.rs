use serde::Serialize;
use serde_derive::Deserialize;
use std::{env, sync::LazyLock};

static HOME_DIRECTORY: LazyLock<String> = LazyLock::new(|| {
    env::var_os("HOME")
        .expect("Fatal, no home directory found")
        .into_string()
        .unwrap()
});

#[derive(Serialize, Deserialize, Debug)]
pub struct UiData {
    pub label: String,
    pub img: String,
    pub serving_size: usize,
    pub ingredients: String,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            label: "Default snack".to_string(),
            img: "caldo-icon-blue.svg".to_string(),
            serving_size: 20,
            ingredients: "Potential allergens if Ichibu is loaded".to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DispenseParameters {
    pub motor_speed: f64,
    pub sample_rate: usize,
    pub cutoff_freq: usize,
    pub check_offset: f64,
    pub stop_offset: usize,
    pub retract_before: bool,
    pub retract_before_param: f64,
    pub retract_after: bool,
    pub retract_after_param: f64,
}

impl Default for DispenseParameters {
    fn default() -> Self {
        Self {
            motor_speed: 0.7,
            sample_rate: 50,
            cutoff_freq: 2,
            check_offset: 0.3,
            stop_offset: 1,
            retract_before: false,
            retract_before_param: 0.0,
            retract_after: false,
            retract_after_param: 0.0,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Ingredient {
    pub name: String,
    pub id: usize,
    pub max_setpoint: usize,
    pub min_setpoint: usize,
    pub ui_data: UiData,
    pub dispense_parameters: DispenseParameters,
}

impl Default for Ingredient {
    fn default() -> Self {
        Self {
            name: "Default Snack".to_string(),
            id: Default::default(),
            max_setpoint: 25,
            min_setpoint: 10,
            ui_data: Default::default(),
            dispense_parameters: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Ingredients {
    pub ingredients: Vec<Ingredient>,
}

impl Default for Ingredients {
    fn default() -> Self {
        let v = vec![Ingredient::default()];
        Self { ingredients: v }
    }
}

pub fn read_ingredient_config() -> Result<Ingredients, Box<dyn std::error::Error>> {
    const PATH: &str = ".config/ichibu/ingredient_config.toml";
    let path = format!("{}/{}", &*HOME_DIRECTORY, PATH);
    println!("{}", path);
    let config_content = std::fs::read_to_string(path)?;
    let config = toml::from_str(&config_content)?;
    Ok(config)
}

pub fn read_image(filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const PATH: &str = ".config/ichibu/images/";
    let path = format!("{}/{}/{}", &*HOME_DIRECTORY, PATH, filename);
    let image = std::fs::read(path)?;
    Ok(image)
}

pub fn read_caldo_logo() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const CALDO_LOGO: &str = "caldo-icon-blue.svg";
    let logo = read_image(CALDO_LOGO)?;
    Ok(logo)
}

#[test]
fn test_read_ingredient_config() {
    let config = read_ingredient_config();
    assert_ne!(config.is_err(), true)
}

#[test]
fn test_read_caldo_logo() {
    let logo = read_caldo_logo();
    assert_ne!(logo.is_err(), true);
    println!("{:?}", logo.unwrap())
}
