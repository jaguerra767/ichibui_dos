use control_components::components::scale::ScaleCmd;
use control_components::subsystems::dispenser::{Parameters, Setpoint, WeightedDispense};
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::config::Config;
use crate::dispense::DispenseHandle;
use crate::hatch::Hatch;
use crate::ingredients::Ingredient;
use crate::io::{initialize_controller, initialize_hatch, PhotoEyeState};
use crate::state::{AppData, IchibuState};
use crate::UiRequest;

pub async fn ichibu_cycle(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser_motor_id: usize,
    scale_tx: Sender<ScaleCmd>,
) {
    let config = Config::load();

    let cc_handle = initialize_controller(&config);

    let dispenser = DispenseHandle::new(cc_handle.get_motor(dispenser_motor_id).clone(), scale_tx);

    let mut hatch = initialize_hatch(&cc_handle, &config).await;

    run_cycle_loop(state, &dispenser, &mut hatch).await;
}

async fn wait_for_pe(state: tauri::State<'_, Mutex<AppData>>) {
    while matches!(
        state.lock().unwrap().get_pe_state(),
        PhotoEyeState::Unblocked
    ) {
        sleep(Duration::from_millis(250)).await;
    }
}

async fn run_cycle_loop(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    hatch: &mut Hatch,
) {
    loop {
        let state = state.clone();
        let (ichibu_state, pe_state) = {
            let state = state.lock().unwrap();
            let ichibu_state = state.get_state();
            let pe_state = state.get_pe_state();
            (ichibu_state, pe_state)
        };
        match ichibu_state {
            IchibuState::Cleaning => dispenser.disable().await,
            IchibuState::Emptying => handle_emptying_state(dispenser, hatch, pe_state).await,
            IchibuState::Ready => todo!(),
            IchibuState::RunningClassic | IchibuState::RunningSized => handle_running_state(state, dispenser, hatch).await,
        }
    }
}

async fn handle_running_state(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    hatch: &mut Hatch,
) {
    //Make sure we don't keep the mutex lock as dispense blocks...

    let snack = { state.lock().unwrap().get_snack().unwrap().clone() };

    let setpoint = {
        let ichibu_state = state.lock().unwrap().get_state();
        if matches!(ichibu_state, IchibuState::RunningClassic) {
            snack.max_setpoint
        } else {
            snack.min_setpoint
        }
    };

    let setpoint = Setpoint::Weight(WeightedDispense {
        setpoint: setpoint as f64,
        timeout: Duration::from_millis(30000),
    });

    let parameters = Parameters::from(&snack.dispense_parameters);

    dispenser.launch_dispense(setpoint, parameters).await;

    handle_user_selection(state.clone(), dispenser, &snack).await;

    wait_for_pe(state.clone()).await;
    hatch.open().await.unwrap();
    sleep(Duration::from_millis(1000)).await;
    hatch.close().await.unwrap();

    let mut state = state.lock().unwrap();
    state.log_dispense();
    state.reset_ui_request();
}

async fn handle_user_selection(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    snack: &Ingredient,
) {
    loop {
        let (request, ichibu_state) = {
            let state = state.lock().unwrap();
            let request = state.get_ui_request();
            let ichibu_state = state.get_state();
            (request, ichibu_state)
        };
        match request {
            UiRequest::None => sleep(Duration::from_millis(250)).await,
            UiRequest::SmallDispense => break,
            UiRequest::RegularDispense => {
                if matches!(ichibu_state, IchibuState::RunningClassic) {
                    let sp = snack.max_setpoint - snack.min_setpoint;
                    let setpoint = Setpoint::Weight(WeightedDispense {
                        setpoint: sp as f64,
                        timeout: Duration::from_micros(1000),
                    });
                    dispenser
                        .launch_dispense(setpoint, Parameters::from(&snack.dispense_parameters))
                        .await;
                    break;
                }
            }
        }
    }
}

async fn handle_emptying_state(
    dispenser: &DispenseHandle,
    hatch: &mut Hatch,
    pe_state: PhotoEyeState,
) {
    hatch.open().await.unwrap();
    if matches!(pe_state, PhotoEyeState::Blocked) {
        dispenser.empty().await;
    } else {
        dispenser.disable().await;
    }
}
