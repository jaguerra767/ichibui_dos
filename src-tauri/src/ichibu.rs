use control_components::components::scale::ScaleCmd;
use control_components::subsystems::dispenser::{
    DispenseEndCondition, Parameters, Setpoint, WeightedDispense,
};
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::config::Config;
use crate::data_logging::DataAction;
use crate::dispense::DispenseHandle;
use crate::hatch::Hatch;
use crate::ingredients::Ingredient;
use crate::io::{initialize_controller, initialize_hatch, setup_conveyor_motor, PhotoEyeState};
use crate::state::{AppData, IchibuState};
use crate::UiRequest;

pub async fn ichibu_cycle(state: tauri::State<'_, Mutex<AppData>>, scale_tx: Sender<ScaleCmd>) {
    let config = Config::load();

    let cc_handle = initialize_controller(&config);
    let motor = setup_conveyor_motor(&config, &cc_handle).await;
    let dispenser = DispenseHandle::new(motor, scale_tx);

    let mut hatch = initialize_hatch(&cc_handle, &config).await;

    let _ = hatch.close().await;

    run_cycle_loop(state, &dispenser, &mut hatch).await;
}

async fn wait_for_pe(state: tauri::State<'_, Mutex<AppData>>) {
    while matches!(
        state.lock().unwrap().get_pe_state(),
        PhotoEyeState::Unblocked
    ) {
        log::info!("Waiting for photoeye input");
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
    sleep(Duration::from_millis(2000)).await;
    log::info!("Starting primary dispense");
    let dispense = dispenser.launch_dispense(setpoint, parameters).await;
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.set_dispenser_busy(false);

        match dispense {
            DispenseEndCondition::WeightAchieved(_) => {
                log::info!("Primary dispense COMPLETE");
            }
            DispenseEndCondition::Timeout(_) => {
                if state_guard.cycle_dispense_count > 2 {
                    log::info!("Oh fuck we timed out");
                    state_guard.dispenser_has_timed_out = true;
                    state_guard.update_state(IchibuState::Ready);
                    let action = DataAction::RanOut;
                    state_guard.log_action(&action);
                    return;
                }
            }
            DispenseEndCondition::Failed => log::error!("Failed to Dispense!"),
        }
    }

    handle_user_selection(state.clone(), dispenser, &snack).await;

    let ichibu_state = {
        let state = state.lock().unwrap();
        let ichibu_state = state.get_state();
        ichibu_state
    };

    if matches!(ichibu_state, IchibuState::Cleaning) {
        let cleaning = DataAction::Cleaning;
        let mut state_guard = state.lock().unwrap();
        state_guard.log_action(&cleaning);
        state_guard.dispenser_has_timed_out = false;
        state_guard.cycle_dispense_count = 0;
        return;
    }
    if matches!(ichibu_state, IchibuState::Emptying) {
        let emptying = DataAction::Emptying;
        let mut state_guard = state.lock().unwrap();
        state_guard.log_action(&emptying);
        state_guard.dispenser_has_timed_out = false;
        state_guard.cycle_dispense_count = 0;
        return;
    }

    if hatch.open().await.is_err() {
        log::error!("Hatch open timed out!");
    }
    sleep(Duration::from_millis(1000)).await;
    let mut state = state.lock().unwrap();
    state.cycle_dispense_count = state.cycle_dispense_count + 1;
    log::info!("Dispense count: {}", state.cycle_dispense_count);
    state.reset_ui_request();
}

async fn handle_user_selection(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    snack: &Ingredient,
) {
    log::info!("Waiting for user input");
    loop {
        let (request, ichibu_state) = {
            let state = state.lock().unwrap();
            let request = state.get_ui_request();
            let ichibu_state = state.get_state();
            (request, ichibu_state)
        };
        if matches!(ichibu_state, IchibuState::Cleaning) {
            let cleaning = DataAction::Cleaning;
            let mut state_guard = state.lock().unwrap();
            state_guard.log_action(&cleaning);
            state_guard.dispenser_has_timed_out = false;
            state_guard.cycle_dispense_count = 0;
            return;
        }
        if matches!(ichibu_state, IchibuState::Emptying) {
            let emptying = DataAction::Emptying;
            let mut state_guard = state.lock().unwrap();
            state_guard.log_action(&emptying);
            state_guard.dispenser_has_timed_out = false;
            state_guard.cycle_dispense_count = 0;
            return;
        }


        match request {
            UiRequest::None => sleep(Duration::from_millis(250)).await,
            UiRequest::SmallDispense => {
                let small_dispense = DataAction::DispensedSmall;
                let mut state_guard = state.lock().unwrap();
                if !state_guard.dispenser_has_timed_out {
                    state_guard.log_action(&small_dispense);
                } else {
                    let action = DataAction::RanOut;
                    state_guard.log_action(&action);
                    state_guard.update_state(IchibuState::Ready);
                }
                break;
            }
            UiRequest::RegularDispense => {
                if matches!(ichibu_state, IchibuState::RunningSized) {
                    let sp = snack.max_setpoint - snack.min_setpoint;
                    let setpoint = Setpoint::Weight(WeightedDispense {
                        setpoint: sp as f64,
                        timeout: Duration::from_micros(1000),
                    });
                    log::info!("Starting secondary dispense");
                    {
                        state.lock().unwrap().set_dispenser_busy(true);
                    }
                    let dispense = dispenser
                        .launch_dispense(setpoint, Parameters::from(&snack.dispense_parameters))
                        .await;

                    let mut state_guard = state.lock().unwrap();
                    state_guard.set_dispenser_busy(false);

                    match dispense {
                        DispenseEndCondition::WeightAchieved(_) => {
                            log::info!("Primary dispense COMPLETE");
                            let action = &DataAction::DispensedRegular;
                            state_guard.log_action(action);
                        }
                        DispenseEndCondition::Timeout(_) => {
                            if state_guard.cycle_dispense_count > 2 {
                                state_guard.dispenser_has_timed_out = true;
                                state_guard.update_state(IchibuState::Ready);
                                let action = DataAction::RanOut;
                                state_guard.log_action(&action);
                                return;
                            }
                        }
                        DispenseEndCondition::Failed => log::error!("Failed to Dispense!"),
                    }
            
                    log::info!("Secondary Dispense COMPLETE");
                } else {
                    let action = &DataAction::DispensedRegular;
                    state.lock().unwrap().log_action(action);
                }
                log::info!("Breaking out of handle_user_selection");
                
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
