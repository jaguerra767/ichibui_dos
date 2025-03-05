// use std::{env, sync::LazyLock};
// use ingredients::{UiData};

use ingredients::{read_caldo_logo, read_image, read_ingredient_config, UiData};
use tauri::ipc::Response;

pub mod ingredients;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_ingredient_data,
            get_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
