use async_clear_core::motor::ClearCoreMotor;


use std::sync::Mutex;
use std::time::Duration;

use tokio::time::sleep;

use crate::config::Config;
use crate::data_logging::DataAction;

use crate::hatch::Hatch;
use crate::ingredients::Ingredient;
use crate::io::{initialize_controller, initialize_hatch, PhotoEyeState};
use crate::state::{AppData, IchibuState};
use crate::UiRequest;

pub async fn ichibu_cycle(state: tauri::State<'_, Mutex<AppData>>) {
    let config = Config::load();

    let cc_handle = initialize_controller(&config);
    let motor_id = config.motor.id;


    let mut hatch = initialize_hatch(&cc_handle, &config).await;

    let _ = hatch.close().await;

    run_cycle_loop(state, cc_handle.get_motor(motor_id),&mut hatch).await;
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
    motor: ClearCoreMotor,
    hatch: &mut Hatch,
) ->anyhow::Result<()> {
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
                motor.abrupt_stop().await?;
                motor.disable().await?;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            IchibuState::Emptying => handle_emptying_state(dispenser, hatch, pe_state).await,
            IchibuState::Ready => tokio::time::sleep(Duration::from_millis(1000)).await,
            IchibuState::RunningClassic | IchibuState::RunningSized => {
                handle_running_state(state, dispenser, hatch).await
            }
        }
    }
    Ok(())
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

    let timed_out = { state.lock().unwrap().dispenser_timed_out() };
    if timed_out {
        println!("Skipping dispense because still timed out!");
        return;
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
    log::info!("Starting primary dispense");
    let dispense_result = dispenser.launch_dispense(setpoint, parameters).await;
    {
        let mut guard = state.lock().unwrap();
        guard.set_dispenser_busy(false);
        if matches!(dispense_result, DispenseEndCondition::Timeout(_)) {
            guard.set_dispenser_timed_out(true);
            return;
        }
    }
    log::info!("Primary dispense COMPLETE");
    handle_user_selection(state.clone(), dispenser, &snack).await;

    let ichibu_state = {
        let state = state.lock().unwrap();
        let ichibu_state = state.get_state();
        ichibu_state
    };
    //Todo: log cleaning mode and empty mode here aka mode transition
    if matches!(ichibu_state, IchibuState::Cleaning) {
        let cleaning = DataAction::Cleaning;
        state.lock().unwrap().log_action(&cleaning);
        return;
    }
    if matches!(ichibu_state, IchibuState::Emptying) {
        let emptying = DataAction::Emptying;
        state.lock().unwrap().log_action(&emptying);
        return;
    }

    if hatch.open().await.is_err() {
        log::error!("Hatch open timed out!");
    }
    sleep(Duration::from_millis(1000)).await;
    let mut state = state.lock().unwrap();
    state.reset_ui_request();
}

async fn handle_user_selection(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    snack: &Ingredient,
) {
    loop {
        log::info!("Waiting for user input");
        let (request, ichibu_state) = {
            let state = state.lock().unwrap();
            let request = state.get_ui_request();
            let ichibu_state = state.get_state();
            (request, ichibu_state)
        };
        if matches!(ichibu_state, IchibuState::Cleaning) {
            let cleaning = DataAction::Cleaning;
            state.lock().unwrap().log_action(&cleaning);
            return;
        }
        if matches!(ichibu_state, IchibuState::Emptying) {
            let emptying = DataAction::Emptying;
            state.lock().unwrap().log_action(&emptying);
            return;
        }

        match request {
            UiRequest::None => sleep(Duration::from_millis(250)).await,
            UiRequest::SmallDispense => {
                let small_dispense = DataAction::DispensedSmall;
                state.lock().unwrap().log_action(&small_dispense);
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
                    dispenser
                        .launch_dispense(setpoint, Parameters::from(&snack.dispense_parameters))
                        .await;
                    {
                        state.lock().unwrap().set_dispenser_busy(false);
                    }
                    log::info!("Secondary Dispense COMPLETE");
                }
                {
                    let regular_dispense = DataAction::DispensedRegular;
                    state.lock().unwrap().log_action(&regular_dispense);
                }
                break;
            }
        }
    }
    wait_for_pe(state.clone()).await;
}

async fn handle_emptying_state(
    motor: ClearCoreMotor,
    hatch: &mut Hatch,
    pe_state: PhotoEyeState,
) -> anyhow::Result<()> {
    if matches!(pe_state, PhotoEyeState::Blocked) {
        if hatch.open().await.is_err() {
            log::error!("Hatch Failed to Open")
        }
        motor.relative_move(1000.).await?;
    } else {
        motor.abrupt_stop().await?;
        motor.disable().await?;
    }
    Ok(())
}
