use async_clear_core::motor::ClearCoreMotor;
use tokio::sync::mpsc;

use std::sync::Mutex;
use std::time::Duration;

use tokio::time::sleep;

use crate::config::Config;
use crate::data_logging::DataAction;

use crate::dispense::{dispense, DispenserIo};
use crate::hatch::Hatch;
use crate::ingredients::Ingredient;
use crate::io::{initialize_controller, initialize_hatch, PhotoEyeState};
use crate::scale::ScaleRequest;
use crate::state::{AppData, IchibuState};
use crate::UiRequest;

pub async fn ichibu_cycle(
    state: tauri::State<'_, Mutex<AppData>>,
    scale_tx: mpsc::Sender<ScaleRequest>,
) {
    let config = Config::load();

    let cc_handle = initialize_controller(&config);
    let motor_id = config.motor.id;

    let mut hatch = initialize_hatch(cc_handle.clone(), &config).await;

    let _ = hatch.close().await;

    let dispenser_io = DispenserIo {
        motor: cc_handle.get_motor(motor_id),
        scale: scale_tx,
    };

    run_cycle_loop(state, dispenser_io, &mut hatch).await;
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
    dispenser_io: DispenserIo,
    hatch: &mut Hatch,
) -> anyhow::Result<()> {
    loop {
        let state = state.clone();
        let (ichibu_state, pe_state) = {
            let state = state.lock().unwrap();
            let ichibu_state = state.get_state();
            let pe_state = state.get_pe_state();
            (ichibu_state, pe_state)
        };

        let io = dispenser_io.clone();
        match ichibu_state {
            IchibuState::Cleaning => {
                if hatch.open().await.is_err() {
                    log::error!("Hatch Failed To Open")
                }
                io.clone().motor.abrupt_stop().await?;
                io.clone().motor.disable().await?;
            }
            IchibuState::Emptying => {
                handle_emptying_state(dispenser_io.clone().motor, hatch, pe_state).await?
            }
            IchibuState::Ready => tokio::time::sleep(Duration::from_millis(10)).await,
            IchibuState::RunningClassic | IchibuState::RunningSized => {
                handle_running_state(state, io.clone(), hatch).await
            }
        }
    }
}

async fn handle_running_state(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser_io: DispenserIo,
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

    let parameters = crate::dispense::Parameters {
        motor_speed: snack.dispense_parameters.motor_speed,
        min_speed: 0.1,
        check_offset: snack.dispense_parameters.check_offset,
        sample_rate: 64.,
        samples: 100,
        reverse_before: {
            if snack.dispense_parameters.retract_before {
                Some(snack.dispense_parameters.retract_before_param)
            } else {
                None
            }
        },
        reverse_after: {
            if snack.dispense_parameters.retract_after {
                Some(snack.dispense_parameters.retract_after_param)
            } else {
                None
            }
        },
    };

    {
        state.lock().unwrap().set_dispenser_busy(true);
    }
    log::info!("Starting primary dispense");
    let dispense_result = dispense(dispenser_io.clone(), setpoint as f64, parameters).await;
    {
        let mut guard = state.lock().unwrap();
        guard.set_dispenser_busy(false);
        if dispense_result.is_err() {
            guard.set_dispenser_timed_out(true);
            return;
        }
    }
    log::info!("Primary dispense COMPLETE");
    handle_user_selection(state.clone(), dispenser_io, &snack).await;

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
    sleep(Duration::from_millis(100)).await;
    let mut state = state.lock().unwrap();
    state.reset_ui_request();
}

async fn handle_user_selection(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser_io: DispenserIo,
    snack: &Ingredient,
) -> anyhow::Result<()> {
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
            return Ok(());
        }
        if matches!(ichibu_state, IchibuState::Emptying) {
            let emptying = DataAction::Emptying;
            state.lock().unwrap().log_action(&emptying);
            return Ok(());
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
                    log::info!("Starting secondary dispense");
                    {
                        state.lock().unwrap().set_dispenser_busy(true);
                    }
                    let parameters = crate::dispense::Parameters {
                        motor_speed: snack.dispense_parameters.motor_speed,
                        min_speed: 0.1,
                        check_offset: snack.dispense_parameters.check_offset,
                        sample_rate: 64.,
                        samples: 100,
                        reverse_before: {
                            if snack.dispense_parameters.retract_before {
                                Some(snack.dispense_parameters.retract_before_param)
                            } else {
                                None
                            }
                        },
                        reverse_after: {
                            if snack.dispense_parameters.retract_after {
                                Some(snack.dispense_parameters.retract_after_param)
                            } else {
                                None
                            }
                        },
                    };
                    let _ = dispense(dispenser_io, sp as f64, parameters).await?;
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
    Ok(())
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
