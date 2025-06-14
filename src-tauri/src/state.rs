use std::sync::Mutex;
use std::time::Duration;
use log::info;
use tokio::sync::{mpsc::Sender, oneshot};

use control_components::components::{clear_core_io::DigitalInput, scale::ScaleCmd};
use serde::{Deserialize, Serialize};

use crate::{
    data_logging::{Data, DataAction},
    ingredients::{read_ingredient_config, Ingredient},
    io::{self, PhotoEyeState},
    UiRequest, HOME_DIRECTORY,
};
use crate::lights::{LightColors, Lights};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum IchibuState {
    #[default]
    Ready,
    RunningClassic,
    RunningSized,
    Cleaning,
    Emptying,
}

#[derive(Default, Debug, Serialize)]
pub enum NodeLevel {
    #[default]
    Filled,
    Empty,
}

//App data is what should be shared between the UI and the controls
pub struct AppData {
    state: IchibuState,
    ui_request: UiRequest,
    node_level: NodeLevel,
    pe_state: io::PhotoEyeState,
    dispenser_busy: bool,
    pub dispenser_has_timed_out: bool,
    database: Data,
    bowl_count: i64,
    pub cycle_dispense_count: usize,
    current_snack: Option<Ingredient>,
}

impl AppData {
    pub fn new() -> Self {
        let (database, bowl_count) = io::initialize_database();
        // let pe_state = io::photo_eye_state(&photo_eye).await;
        Self {
            state: IchibuState::Ready,
            ui_request: UiRequest::None,
            node_level: NodeLevel::Empty,
            pe_state: PhotoEyeState::Unblocked,
            dispenser_busy: false,
            dispenser_has_timed_out: false,
            database,
            bowl_count,
            cycle_dispense_count: 0,
            current_snack: None,
        }
    }

    pub fn log_action(&mut self, action: &DataAction) {
        let snack_id = self.current_snack.as_ref().map(|snack| snack.id);
        let _ = self.database.log(action, snack_id);
        self.bowl_count = self.database.get_bowl_count().unwrap();
    }

    pub fn reset_ui_request(&mut self) {
        self.ui_request = UiRequest::None;
    }

    pub fn get_state(&self) -> IchibuState {
        self.state.clone()
    }

    pub fn get_pe_state(&self) -> PhotoEyeState {
        self.pe_state.clone()
    }

    pub fn get_snack(&self) -> Option<&Ingredient> {
        self.current_snack.as_ref()
    }

    pub fn get_ui_request(&self) -> UiRequest {
        self.ui_request.clone()
    }

    pub fn set_dispenser_busy(&mut self, is_busy: bool) {
        self.dispenser_busy = is_busy;
    }

    pub fn update_state(&mut self, new_state: IchibuState) {
        self.state = new_state;
    }
    //These are private so that they can only be called from the UI via the tauri commands below
    fn update_current_snack(&mut self, snack: Ingredient) {
        self.current_snack = Some(snack);
    }

    fn update_ui_request(&mut self, ui_request: UiRequest) {
        self.ui_request = ui_request;
    }

    fn dispenser_is_busy(&self) -> bool {
        self.dispenser_busy
    }
}

//These are so that we can have a task updating these
pub async fn update_pe_state(state: tauri::State<'_, Mutex<AppData>>, photo_eye: DigitalInput) {
    let pe_state = io::photo_eye_state(&photo_eye).await;
    state.lock().unwrap().pe_state = pe_state;
}
pub async fn update_lights_state(state: tauri::State<'_, Mutex<AppData>>, mut lights: Lights, interval: Duration) {
    let (run_state, dispenser_busy, is_timed_out) = {
        let state = state.lock().unwrap();
        let run_state = state.get_state();
        let dispenser_busy = state.dispenser_is_busy();
        let is_timed_out = state.dispenser_has_timed_out;
        (run_state, dispenser_busy, is_timed_out)
    };
    match (run_state, dispenser_busy, is_timed_out) {
        (_, _, true) => {
            lights.set_color(LightColors::Red).await;
            tokio::time::sleep(interval/2).await;
            lights.turn_off().await;
        }
        (IchibuState::Ready, _, _) => {
            lights.turn_off().await;
        }
        (IchibuState::RunningClassic | IchibuState::RunningSized, true, _) => {
            lights.set_color(LightColors::Yellow).await;
        }
        (IchibuState::RunningClassic | IchibuState::RunningSized, false, _) => {
            lights.set_color(LightColors::Green).await;
        }
        _ => {
            lights.turn_off().await;
        }
    }
}
pub async fn update_node_level(
    state: tauri::State<'_, Mutex<AppData>>,
    empty_weight: f64,
    scale_tx: Sender<ScaleCmd>,
) {
    let (send, recv) = oneshot::channel();
    let msg = ScaleCmd(send);
    let _ = scale_tx.send(msg).await;
    if let Ok(weight) = recv.await {
        let node_level = if weight > empty_weight {
            NodeLevel::Filled
        } else {
            NodeLevel::Empty
        };
        state.lock().unwrap().node_level = node_level;
    }
}

#[tauri::command]
pub fn update_current_ingredient(state: tauri::State<'_, Mutex<AppData>>, snack: usize) {
    if let Ok(res) = read_ingredient_config(HOME_DIRECTORY.as_str()) {
        if let Some(ingredient) = res.ingredients.iter().find(|ing| ing.id == snack) {
            state
                .lock()
                .unwrap()
                .update_current_snack(ingredient.clone());
        }
    }
}

#[tauri::command]
pub fn update_run_state(state: tauri::State<'_, Mutex<AppData>>, new_state: IchibuState) {
    let mut state_guard = state.lock().unwrap();
    if matches!(state_guard.get_state(), IchibuState::Ready) {
        match new_state {
            IchibuState::Cleaning => {
                let action = DataAction::Cleaning;
                state_guard.log_action(&action)
            }
            IchibuState::Emptying => {
                let action = DataAction::Emptying;
                state_guard.log_action(&action);
            }
            _ => (),
        }
    }
    state_guard.update_state(new_state);
}

#[tauri::command]
pub fn update_ui_request(state: tauri::State<'_, Mutex<AppData>>, ui_request: UiRequest) {
    state.lock().unwrap().update_ui_request(ui_request);
}

#[tauri::command]
pub fn get_dispense_count(state: tauri::State<'_, Mutex<AppData>>) -> usize {
    state.lock().unwrap().bowl_count as usize
}

#[tauri::command]
pub fn get_pe_blocked(state: tauri::State<'_, Mutex<AppData>>) -> bool {
    match state.lock().unwrap().pe_state {
        PhotoEyeState::Blocked => true,
        PhotoEyeState::Unblocked => false,
    }
}

#[tauri::command]
pub fn dispenser_is_busy(state: tauri::State<'_, Mutex<AppData>>) -> bool {
    state.lock().unwrap().dispenser_is_busy()
}

#[tauri::command]
pub fn dispenser_has_timed_out(state: tauri::State<'_, Mutex<AppData>>) -> bool {
    state.lock().unwrap().dispenser_has_timed_out
}

#[tauri::command]
pub fn clear_dispenser_time_out(state: tauri::State<'_, Mutex<AppData>>) {
    info!("Dispenser state cleared");
    let mut state_guard = state.lock().unwrap();
    state_guard.dispenser_has_timed_out = false;
    state_guard.cycle_dispense_count = 0;
    let action = DataAction::Refilled;
    state_guard.log_action(&action);
}
