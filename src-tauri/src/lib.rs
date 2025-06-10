use config::Config;
use control_components::components::scale::actor;
use control_components::components::scale::Scale;
use ichibu::ichibu_cycle;
use ingredients::{read_ingredient_config, UiData};
use io::initialize_controller;
use log::info;
use serde::{Deserialize, Serialize};
use state::clear_dispenser_time_out;
use state::dispenser_has_timed_out;
use state::get_pe_blocked;
use state::update_node_level;
use state::update_pe_state;
use state::{
    dispenser_is_busy, get_dispense_count, update_current_ingredient, update_run_state,
    update_ui_request,
};
use std::env;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use tauri::{ipc::Response, Manager};
use tokio::sync::mpsc::channel;
use libra::scale;

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
    if let Ok(pin_num) = pin.parse::<usize>() {
        if pin_num == pins.sudo {
            std::process::exit(0x0)
        }
       if pin_num == pins.manager {
            println!("Manager, what are we going to dispense today?");
            User::Manager
        } else if pin_num == pins.operator {
            println!("Operator, lets cook");
            User::Operator
        } else {
            User::None
        }
    } else {
        User::None
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load();
    let controller = initialize_controller(&config);

    let photo_eye = controller.get_digital_input(config.photo_eye.input_id);

    tauri::Builder::default()
        .manage(Mutex::new(state::AppData::new()))
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.app_handle();

            //Let's spawn the scale

            // let (scale_tx, scale_rx) = channel(10);
            let scale = scale::DisconnectedScale::new(config.phidget.sn);
            let scale = scale.connect(0., config.phidget.coefficients, Duration::from_secs(10)).expect("Couldn't connect scale!");

            // tauri::async_runtime::spawn({
            //     async move {
            //         let mut scale = Scale::new(config.phidget.sn);
            //         scale = Scale::change_coefficients(scale, coefficients.to_vec());
            //         if let Ok(scale) = scale.connect() {
            //             if let Err(e) = actor(scale, scale_rx).await {
            //                 log::error!("Scale runtime error: {}", e);
            //             }
            //         } else {
            //             log::warn!("Launching in demo mode");
            //         }
            //     }
            // });

            // let empty_weight = config.setpoint.empty;
            // //Routine to update io members of state that we need for the UI
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                // let scale_tx = scale_tx.clone();
                async move {
                    loop {
                        if let Some(state) = app_handle.try_state::<Mutex<state::AppData>>() {
                            // update_node_level(state.clone(), empty_weight, scale_tx.clone())
                            //         .await;
                            update_pe_state(state, photo_eye.clone()).await;
                        }
                        // Add a small delay between updates
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
            });

            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                // let scale_tx = scale_tx.clone();
                async move {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    let state = loop {
                        //wait for state to become available
                        if let Some(state) = app_handle.try_state::<Mutex<state::AppData>> ()  {    
                            break state;
                        } else {
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    };
                    ichibu_cycle(state, scale).await;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_ingredient_data,
            get_image,
            get_dispense_count,
            get_pe_blocked,
            update_current_ingredient,
            update_run_state,
            update_ui_request,
            log_in,
            set_fullscreen,
            dispenser_is_busy,
            dispenser_has_timed_out,
            clear_dispenser_time_out,
            escape
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

#[tauri::command]
fn set_fullscreen(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // Get the main window
        window.set_fullscreen(true).map_err(|e| e.to_string())?;
    } else {
        return Err("Failed to get main window".to_string());
    }
    Ok(())
}

#[tauri::command]
fn escape() {
    info!("Exiting app");
    std::process::exit(0x0);
}
