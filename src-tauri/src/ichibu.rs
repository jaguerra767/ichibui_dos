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

pub async fn ichibu_cycle(state: tauri::State<'_, Mutex<AppData>>, scale_tx: Sender<ScaleCmd>) {
    let config = Config::load();

    let cc_handle = initialize_controller(&config);
    let motor_id = config.motor.id;
    let dispenser = DispenseHandle::new(cc_handle.get_motor(motor_id).clone(), scale_tx);

    let mut hatch = initialize_hatch(&cc_handle, &config).await;

    let _ = hatch.close().await;
    println!("Starting cycle loop");
    run_cycle_loop(state, &dispenser, &mut hatch).await;
}

async fn wait_for_pe(state: tauri::State<'_, Mutex<AppData>>) {
    while matches!(
        state.lock().unwrap().get_pe_state(),
        PhotoEyeState::Unblocked
    ) {
        sleep(Duration::from_millis(250)).await;
    }
    println!("Photoeye Blocked!");
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
            IchibuState::Cleaning => {
                if hatch.open().await.is_err() {
                    log::error!("Hatch Failed To Open")
                }
                dispenser.disable().await;
                tokio::time::sleep(Duration::from_millis(1000)).await
            }
            IchibuState::Emptying => handle_emptying_state(dispenser, hatch, pe_state).await,
            IchibuState::Ready => tokio::time::sleep(Duration::from_millis(1000)).await,
            IchibuState::RunningClassic | IchibuState::RunningSized => {
                handle_running_state(state, dispenser, hatch).await
            }
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
    if hatch.close().await.is_err() {
        log::error!("Hatch Failed to Close");
    }

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

    {
        state.lock().unwrap().set_dispenser_busy(true);
    }
    dispenser.launch_dispense(setpoint, parameters).await;
    {
        state.lock().unwrap().set_dispenser_busy(false);
    }

    handle_user_selection(state.clone(), dispenser, &snack).await;

    let ichibu_state = {
        let state = state.lock().unwrap();
        let ichibu_state = state.get_state();
        ichibu_state
    };

    if matches!(ichibu_state, IchibuState::Cleaning) {
        return;
    }
    if matches!(ichibu_state, IchibuState::Emptying) {
        return;
    }

    if hatch.open().await.is_err() {
        log::error!("Hatch open timed out!");
    }
    sleep(Duration::from_millis(1000)).await;
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
        if matches!(ichibu_state, IchibuState::Cleaning) {
            return;
        }
        if matches!(ichibu_state, IchibuState::Emptying) {
            return;
        }
        match request {
            UiRequest::None => sleep(Duration::from_millis(250)).await,
            UiRequest::SmallDispense => break,
            UiRequest::RegularDispense => {
                if matches!(ichibu_state, IchibuState::RunningSized) {
                    let sp = snack.max_setpoint - snack.min_setpoint;
                    let setpoint = Setpoint::Weight(WeightedDispense {
                        setpoint: sp as f64,
                        timeout: Duration::from_micros(1000),
                    });
                    {
                        state.lock().unwrap().set_dispenser_busy(true);
                    }

                    dispenser
                        .launch_dispense(setpoint, Parameters::from(&snack.dispense_parameters))
                        .await;
                    {
                        state.lock().unwrap().set_dispenser_busy(false);
                    }
                }
                break;
            }
        }
    }

    wait_for_pe(state.clone()).await;
}

async fn handle_emptying_state(
    dispenser: &DispenseHandle,
    hatch: &mut Hatch,
    pe_state: PhotoEyeState,
) {
    if matches!(pe_state, PhotoEyeState::Blocked) {
        if hatch.open().await.is_err() {
            log::error!("Hatch Failed to Open")
        }
        dispenser.empty().await;
    } else {
        dispenser.disable().await;
    }
}
