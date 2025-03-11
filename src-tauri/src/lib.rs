use config::Config;
use control_components::components::scale::actor;
use control_components::components::scale::Scale;
use ingredients::{read_ingredient_config, UiData};
use io::initialize_controller;
use serde::{Deserialize, Serialize};
use state::update_node_level;
use state::update_pe_state;
use state::{get_dispense_count, update_current_ingredient, update_run_state, update_ui_request};
use std::env;
use std::sync::{LazyLock, Mutex};
use tauri::{ipc::Response, Manager};
use tokio::sync::mpsc::channel;

pub mod config;
pub mod data_logging;
pub mod dispense;
pub mod hatch;
pub mod ichibu;
pub mod ingredients;
pub mod io;


pub mod state;

pub static HOME_DIRECTORY: LazyLock<String> = LazyLock::new(|| {
    env::var_os("HOME")
        .expect("Fatal, no home directory found")
        .into_string()
        .unwrap()
});

#[derive(Serialize, Deserialize, Default, Debug)]
pub enum DispenseType {
    #[default]
    Classic,
    LargeSmall,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum UiRequest {
    #[default]
    None,
    SmallDispense,
    RegularDispense,
}

#[derive(Serialize, Deserialize, Default)]
pub enum User {
    #[default]
    None,
    Admin,
    Manager,
    Operator,
}

#[tauri::command]
fn get_ingredient_data() -> Vec<UiData> {
    let response = match read_ingredient_config(HOME_DIRECTORY.as_str()) {
        Ok(data) => data.ingredients.into_iter().map(|i| i.ui_data).collect(),
        Err(_) => vec![UiData::default()],
    };
    response
}

#[tauri::command]
fn get_image(filename: String) -> Response {
    let response = match read_image(HOME_DIRECTORY.as_str(), &filename) {
        Ok(res) => res,
        Err(_) => read_caldo_logo(HOME_DIRECTORY.as_str()).unwrap_or_default(),
    };
    tauri::ipc::Response::new(response)
}

#[tauri::command]
fn log_in(pin: String) -> User {
    let pins = Config::load().pins;
    match pin.parse::<usize>() {
        Ok(pin_num) => match pin_num {
            num if num == pins.sudo => User::Admin,
            num if num == pins.manager => User::Manager,
            num if num == pins.operator => User::Operator,
            _ => User::None,
        },
        Err(_) => User::None,
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load();
    let controller = initialize_controller(&config);

    let photo_eye = controller.get_digital_input(config.photo_eye.input_id);

    tauri::Builder::default()
        .manage(Mutex::new(state::AppData::new(photo_eye.clone())))
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.app_handle();

            let coefficients = config.phidget.coefficients;

            //Lets spawn the scale

            let (scale_tx, scale_rx) = channel(10);

            let mut scale = Scale::new(config.phidget.sn);
            tauri::async_runtime::spawn({
                async move {
                    scale = Scale::change_coefficients(scale, coefficients.to_vec());
                    let scale = scale.connect().unwrap();
                    let _ = actor(scale, scale_rx).await;
                }
            });

            let empty_weight = config.setpoint.empty;
            //Routine to update io members of state that we need for the UI
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                async move {
                    loop {
                        let state = app_handle.state::<Mutex<state::AppData>>();
                        update_node_level(state.clone(), empty_weight, scale_tx.clone()).await;
                        update_pe_state(state, photo_eye.clone()).await;
                    }
                }
            });

            tauri::async_runtime::spawn(async move {
                //Existing Ichibu-os code runs here aka Controls
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_ingredient_data,
            get_image,
            get_dispense_count,
            update_current_ingredient,
            update_run_state,
            update_ui_request,
            log_in
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn read_image(root_dir: &str, filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const PATH: &str = ".config/ichibu/images/";
    let path = format!("{}/{}/{}", root_dir, PATH, filename);
    let image = std::fs::read(path)?;
    Ok(image)
}

pub fn read_caldo_logo(root_dir: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const CALDO_LOGO: &str = "caldo-icon-blue.svg";
    let logo = read_image(root_dir, CALDO_LOGO)?;
    Ok(logo)
}

#[test]
fn test_read_caldo_logo() {
    let logo = read_caldo_logo(HOME_DIRECTORY.as_str());
    assert_ne!(logo.is_err(), true);
    println!("{:?}", logo.unwrap())
}
