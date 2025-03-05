// use std::{env, sync::LazyLock};
// use ingredients::{UiData};

use std::sync::{Arc, Mutex};

use ichibu::{DispenseType, RunState};
use ingredients::{read_caldo_logo, read_image, read_ingredient_config, UiData};
use serde::{Deserialize, Serialize};
use tauri::ipc::Response;

pub mod config;
pub mod data_logging;
pub mod hatch;
pub mod ichibu;
pub mod ingredients;

#[derive(Serialize, Deserialize, Default)]
pub enum User {
    #[default]
    None,
    Admin,
    Manager,
    Operator,
}


#[derive(Default)]
pub struct AppData {
    pub current_ingredient_id: usize,
    pub user: User,
    pub machine_state: ichibu::State,
}

struct IchibuState(Arc<Mutex<AppData>>);


#[tauri::command]
fn get_ingredient_data() -> Vec<UiData> {
    let response = match read_ingredient_config() {
        Ok(data) => data.ingredients.into_iter().map(|i| i.ui_data).collect(),
        Err(_) => vec![UiData::default()],
    };
    response
}

#[tauri::command]
fn get_image(filename: String) -> Response {
    let response = match read_image(&filename) {
        Ok(res) => res,
        Err(_) => read_caldo_logo().unwrap_or_default(),
    };
    tauri::ipc::Response::new(response)
}

#[tauri::command]
fn get_dispense_count(state: tauri::State<'_, IchibuState>) -> usize {
    let state_guard = state.0.lock().unwrap();
    state_guard.machine_state.bowl_count
}

#[tauri::command]
fn update_current_ingredient(state: tauri::State<'_, IchibuState>, snack_id: usize) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.current_ingredient_id = snack_id;

}

#[tauri::command]
fn update_run_state(state: tauri::State<'_, IchibuState>, run_state: RunState) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.machine_state.run_state = run_state
}

#[tauri::command]
fn update_dispense_type(state: tauri::State<'_, IchibuState>, dispense_type: DispenseType){
    let mut state_guard = state.0.lock().unwrap();
    state_guard.machine_state.dispense_type = dispense_type
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    

    tauri::Builder::default()
        .manage(IchibuState(Arc::new(Mutex::new(AppData::default()))))
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                //Existing Ichibu-os code runs here
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_ingredient_data,
            get_image,
            get_dispense_count,
            update_current_ingredient,
            update_run_state,
            update_dispense_type
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
