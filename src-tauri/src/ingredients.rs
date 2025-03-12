use control_components::subsystems::dispenser::Parameters;
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UiData {
    pub id: usize,
    pub label: String,
    pub img: String,
    pub serving_size: usize,
    pub ingredients: String,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            id: 0,
            label: "Default snack".to_string(),
            img: "caldo-icon-blue.svg".to_string(),
            serving_size: 20,
            ingredients: "Potential allergens if Ichibu is loaded".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DispenseParameters {
    pub motor_speed: f64,
    pub sample_rate: f64,
    pub cutoff_freq: f64,
    pub check_offset: f64,
    pub stop_offset: f64,
    pub retract_before: bool,
    pub retract_before_param: f64,
    pub retract_after: bool,
    pub retract_after_param: f64,
}

impl Default for DispenseParameters {
    fn default() -> Self {
        Self {
            motor_speed: 0.7,
            sample_rate: 50.,
            cutoff_freq: 2.,
            check_offset: 0.3,
            stop_offset: 1.,
            retract_before: false,
            retract_before_param: 0.0,
            retract_after: false,
            retract_after_param: 0.0,
        }
    }
}

impl From<&DispenseParameters> for Parameters {
    fn from(value: &DispenseParameters) -> Self {
        let retract_before = if value.retract_before {
            Some(value.retract_before_param)
        } else {
            None
        };

        let retract_after = if value.retract_after {
            Some(value.retract_after_param)
        } else {
            None
        };

        Self {
            motor_speed: value.motor_speed,
            sample_rate: value.sample_rate,
            cutoff_frequency: value.cutoff_freq,
            check_offset: value.check_offset,
            stop_offset: value.stop_offset,
            retract_before,
            retract_after,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
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

pub fn read_ingredient_config(root_dir: &str) -> Result<Ingredients, Box<dyn std::error::Error>> {
    const PATH: &str = ".config/ichibu/ingredient_config.toml";
    let path = format!("{}/{}", root_dir, PATH);
    let config_content = std::fs::read_to_string(path)?;
    let config = toml::from_str(&config_content)?;
    Ok(config)
}

#[test]
fn test_read_ingredient_config() {
    use crate::HOME_DIRECTORY;
    let config = read_ingredient_config(HOME_DIRECTORY.as_str());
    if config.is_err(){
        println!("{:?}", config);
    }
    assert_ne!(config.is_err(), true)
}
