// use std::{env, sync::LazyLock};
// use ingredients::{UiData};

use ichibu::IchibuState;
use ingredients::{read_ingredient_config, Ingredient, UiData};
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::{LazyLock, Mutex},
};
use tauri::{ipc::Response, Manager};

pub mod config;
pub mod data_logging;
pub mod dispense;
pub mod hatch;
pub mod ichibu;
pub mod ingredients;
pub mod io;
pub mod photo_eye;

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
pub enum RunState {
    #[default]
    Ready,
    Running,
    Cleaning,
    Emptying,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum UiRequest {
    #[default]
    None,
    SmallDispense,
    RegularDispense,
}

#[derive(Default, Debug, Serialize)]
pub enum NodeLevel {
    #[default]
    Filled,
    Empty,
}

#[derive(Serialize, Deserialize, Default)]
pub enum User {
    #[default]
    None,
    Admin,
    Manager,
    Operator,
}

#[derive(Default)]
//App data is what should be shared between the UI and the controls
pub struct AppData {
    pub run_state: RunState,
    pub ichibu_state: IchibuState,
    pub ui_request: UiRequest,
    pub dispense_type: DispenseType,
    pub node_level: NodeLevel,
    pub photo_eye_state: io::PhotoEyeState,
    pub bowl_count: usize,
    pub current_ingredient: Option<Ingredient>,
}

impl AppData {
    pub fn new(photo_eye_state: io::PhotoEyeState, bowl_count: usize) -> Self {
        Self {
            run_state: RunState::Ready,
            ichibu_state: IchibuState::Setup,
            ui_request: UiRequest::None,
            dispense_type: DispenseType::Classic,
            node_level: NodeLevel::Empty,
            photo_eye_state,
            bowl_count,
            current_ingredient: None,
        }
    }
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
fn get_dispense_count(state: tauri::State<'_, Mutex<AppData>>) -> usize {
    let state_guard = state.lock().unwrap();
    state_guard.bowl_count
}

#[tauri::command]
fn update_current_ingredient(state: tauri::State<'_, Mutex<AppData>>, snack: Ingredient) {
    let mut state_guard = state.lock().unwrap();
    state_guard.current_ingredient = Some(snack);
}

#[tauri::command]
fn update_run_state(state: tauri::State<'_, Mutex<AppData>>, run_state: RunState) {
    let mut state_guard = state.lock().unwrap();
    state_guard.run_state = run_state
}

#[tauri::command]
fn update_dispense_type(state: tauri::State<'_, Mutex<AppData>>, dispense_type: DispenseType) {
    let mut state_guard = state.lock().unwrap();
    state_guard.dispense_type = dispense_type
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel(1);
    // let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel(1);

    tauri::Builder::default()
        .manage(Mutex::new(AppData::default()))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = app.state::<Mutex<AppData>>().clone();
            tauri::async_runtime::spawn(async move {
                //Existing Ichibu-os code runs here
                let _ = state;
                let config = config::Config::load();
                let _io_handle = io::launch_io(&config).await;
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
