use node_diagnostics::dispenser::DispenseSettings;
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
pub struct Ingredient {
    pub name: String,
    pub id: usize,
    pub max_setpoint: usize,
    pub min_setpoint: usize,
    pub ui_data: UiData,
    pub dispense_settings: DispenseSettings,
}

impl Default for Ingredient {
    fn default() -> Self {
        Self {
            name: "Default Snack".to_string(),
            id: Default::default(),
            max_setpoint: 25,
            min_setpoint: 10,
            ui_data: Default::default(),
            dispense_settings: Default::default(),
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
    // const PATH: &str = "Documents/ingredient_config.toml";
    let path = format!("{}/{}", root_dir, PATH);
    let config_content = std::fs::read_to_string(path)?;
    let config = toml::from_str(&config_content)?;
    Ok(config)
}

#[test]
fn test_read_ingredient_config() {
    use crate::HOME_DIRECTORY;
    let config = read_ingredient_config(HOME_DIRECTORY.as_str());
    if config.is_err() {
        println!("{:?}", config);
    }
    assert!(config.is_ok())
}
