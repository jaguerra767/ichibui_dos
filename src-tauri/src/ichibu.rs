// use crate::config::{self, Config};
// use crate::data_logging::{Data, DataAction};
// use crate::dispense::DispenseHandle;
// use crate::hatch::Hatch;
// use crate::io::{self, setup_conveyor_motor, setup_hatch, PhotoEyeState};
// use crate::AppData;
// use control_components::components::clear_core_io::DigitalInput;
// use control_components::components::clear_core_motor::ClearCoreMotor;
// use control_components::components::scale::ScaleCmd;
// use control_components::controllers::clear_core::{Controller, MotorBuilder};
// use control_components::subsystems::dispenser::{
//     Dispenser, Parameters, Setpoint, WeightedDispense,
// };
// use log::{debug, error, info, warn};
// use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use std::sync::Mutex;
// use std::time::Duration;
// use tokio::sync::mpsc::Sender;
// use tokio::time::Interval;

// type CCController = Controller;
// #[derive(Debug, Deserialize)]
// pub struct CustomIngredient {
//     pub serving_size: f64,
//     pub speed: f64,
// }

// // //Ichibu Controls "entry point"
// // pub async fn cycle() {
// //     let config = Config::load();

// //     let motor_builder = MotorBuilder{ id: config.motor.id as u8, scale: config.motor.scale };
// //     let (c1, c2) = Controller::with_client(config.addresses.clear_core, &[motor_builder]);

// //     let dispense_handle = DispenseHandle::new(motor, phidget_id, coefficients)
// //     loop {

// //     }
// // }

// pub async fn _cycle(
//     app_data: &AppData,
//     config: &Config,
//     motor: ClearCoreMotor,
//     mut hatch: Hatch,
//     io: Sender<ScaleCmd>,
// ) {
//     let mut parameters = Parameters::default();
//     let mut setpoint = Setpoint::Weight(WeightedDispense {
//         setpoint: 0.0,
//         timeout: Duration::from_micros(1000),
//     });

//     match app_data.run_state {
//         RunState::Ready => {
//             hatch.close().await.unwrap();
//             if let Err(e) = motor.enable().await {
//                 error!("Failed to enable motor: {:?}", e);
//             }
//             if let Some(snack) = &app_data.current_ingredient {
//                 setpoint = Setpoint::Weight(WeightedDispense {
//                     setpoint: snack.min_setpoint as f64,
//                     timeout: Duration::from_micros(1000),
//                 });

//                 let retract_before = if snack.dispense_parameters.retract_before {
//                     Some(snack.dispense_parameters.retract_before_param)
//                 } else {
//                     None
//                 };

//                 let retract_after = if snack.dispense_parameters.retract_after {
//                     Some(snack.dispense_parameters.retract_after_param)
//                 } else {
//                     None
//                 };

//                 parameters = Parameters {
//                     motor_speed: snack.dispense_parameters.motor_speed,
//                     sample_rate: snack.dispense_parameters.sample_rate,
//                     cutoff_frequency: snack.dispense_parameters.cutoff_freq,
//                     check_offset: snack.dispense_parameters.check_offset,
//                     stop_offset: snack.dispense_parameters.stop_offset,
//                     retract_before,
//                     retract_after,
//                 };
//             }
//         }
//         RunState::Running => {
//             if let Some(snack) = &app_data.current_ingredient {
//                 let dispenser =
//                     Dispenser::new(motor.clone(), setpoint, parameters.clone(), io.clone());

//                 dispenser.dispense(config.dispense.timeout).await;

//                 //Here we wait for dispense button to be pressed
//                 if matches!(app_data.dispense_type, DispenseType::LargeSmall) {
//                     let setpoint = snack.max_setpoint - snack.min_setpoint;
//                     let setpoint = Setpoint::Weight(WeightedDispense {
//                         setpoint: setpoint as f64,
//                         timeout: Duration::from_micros(1000),
//                     });
//                     Dispenser::new(motor.clone(), setpoint, parameters, io)
//                         .dispense(config.dispense.timeout)
//                         .await;
//                 }
//             }
//         }
//         RunState::Cleaning => motor.disable().await,
//         RunState::Emptying => todo!(),
//     }
// }

// async fn photo_eye_state(input: DigitalInput) -> PhotoEyeState {
//     if input.get_state().await {
//         PhotoEyeState::Blocked
//     } else {
//         PhotoEyeState::Unblocked
//     }
// }

// async fn wait_for_photoeye(input: DigitalInput) {
//     if matches!(photo_eye_state(input).await, PhotoEyeState::Unblocked) {
//         loop {
//             tokio::time::sleep(Duration::from_millis(250)).await;
//         }
//     }
// }

use control_components::components::clear_core_io::DigitalInput;
use control_components::controllers::clear_core::{Controller, MotorBuilder};
use control_components::subsystems::dispenser::{Parameters, Setpoint, WeightedDispense};
use rusqlite::Connection;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::Config;
use crate::data_logging::{Data, DataAction};
use crate::dispense::{self, DispenseHandle};
use crate::hatch::Hatch;
use crate::ingredients::Ingredient;
use crate::io::{photo_eye_state, PhotoEyeState};
use crate::{photo_eye, AppData, RunState, UiRequest, HOME_DIRECTORY};

const DB_PATH: &str = ".config/ichibu/data/data.db";

#[derive(Default)]
pub enum IchibuState {
    #[default]
    Setup,
    Ready,
    Dispensing,
    DispenseComplete,
    OpeningHatch,
    ClosingHatch,
}

// pub async fn ichibu_cycle(state: tauri::State<'_, Mutex<AppData>>) {
//     let config = Config::load();

//     // Initialize components
//     let (database, bowl_count) = initialize_database();
//     let (cc_handle, cc_client) = initialize_controller(&config);
//     let dispenser = initialize_dispenser(&cc_handle, &config);
//     let mut hatch = initialize_hatch(&cc_handle, &config).await;
//     let photo_eye = cc_handle.get_digital_input(config.photo_eye.input_id);

//     // Update UI with bowl count
//     update_ui_bowl_count(state.clone(), bowl_count);

//     // Spawn Controller client
//     tauri::async_runtime::spawn(cc_client);

//     // Main loop
//     run_cycle_loop(state, &dispenser, &mut hatch, &photo_eye, &database, bowl_count).await;
// }

pub async fn ichibu_cycle(state: tauri::State<'_, Mutex<AppData>>) {
    let config = Config::load();
    let (data, bowl_count) = initialize_database();
    let cc_handle = initialize_controller(&config);
    let dispenser = initialize_dispenser(&cc_handle, &config);
    let mut hatch = initialize_hatch(&cc_handle, &config).await;
    let photo_eye = cc_handle.get_digital_input(config.photo_eye.input_id);

    update_ui_bowl_count(state.clone(), bowl_count); 
    run_cycle_loop(state, &dispenser, &mut hatch, &photo_eye, &data, bowl_count).await;
}

fn update_ui_bowl_count(state: tauri::State<'_, Mutex<AppData>>, count: i64) {
    state.lock().unwrap().bowl_count = count as usize;
}

fn update_photoeye_state(state: tauri::State<'_, Mutex<AppData>>, pe_state: PhotoEyeState) {
    state.lock().unwrap().photo_eye_state = pe_state;
}

fn reset_ui_request(state: tauri::State<'_, Mutex<AppData>>) {
    state.lock().unwrap().ui_request = UiRequest::None;
}
fn get_ui_state(state: tauri::State<'_, Mutex<AppData>>) -> RunState {
    state.lock().unwrap().run_state.clone()
}

fn get_snack(state: tauri::State<'_, Mutex<AppData>>) -> Option<Ingredient> {
    state.lock().unwrap().current_ingredient.clone()
}

fn get_ui_request(state: tauri::State<'_, Mutex<AppData>>) -> UiRequest {
    state.lock().unwrap().ui_request.clone()
}

async fn wait_for_pe(pe: &DigitalInput) {
    while matches!(photo_eye_state(pe).await, PhotoEyeState::Unblocked) {
        sleep(Duration::from_millis(250)).await;
    }
}

// ---- Initialization Functions ----

fn initialize_database() -> (Data, i64) {
    let database_path = format!("{}/{}", HOME_DIRECTORY.as_str(), DB_PATH);
    let database_connection = Connection::open(database_path).unwrap();
    let database = Data::new(database_connection);
    let bowl_count = database.connect().unwrap();
    (database, bowl_count)
}

fn initialize_controller(config: &Config) -> Controller {
    let (controller, controller_client) = Controller::with_client(
        config.addresses.clear_core.clone(),
        &[
            MotorBuilder {
                id: config.motor.id as u8,
                scale: config.motor.scale,
            },
            MotorBuilder {
                id: config.hatch.motor_id as u8,
                scale: 200,
            },
        ],
    );
    tauri::async_runtime::spawn(controller_client);
    controller
}

fn initialize_dispenser(cc_handle: &Controller, config: &Config) -> DispenseHandle {
    DispenseHandle::new(
        cc_handle.get_motor(config.motor.id),
        config.phidget.sn,
        &config.phidget.coefficients,
    )
}

async fn initialize_hatch(cc_handle: &Controller, config: &Config) -> Hatch {
    let mut hatch = Hatch::new(
        cc_handle.get_motor(config.hatch.motor_id),
        cc_handle.get_digital_input(config.hatch.open_input),
        cc_handle.get_digital_input(config.hatch.close_input),
    );
    hatch.setup().await;
    hatch
}

async fn run_cycle_loop(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    hatch: &mut Hatch,
    photo_eye: &DigitalInput,
    database: &Data,
    mut bowl_count: i64,
) {
    loop {
        let pe_state = photo_eye_state(photo_eye).await;
        update_photoeye_state(state.clone(), pe_state.clone());

        match get_ui_state(state.clone()) {
            RunState::Ready => continue,
            RunState::Running => handle_running_state(state.clone(), dispenser, hatch, photo_eye, database, &mut bowl_count).await,
            RunState::Cleaning => dispenser.disable().await,
            RunState::Emptying => handle_emptying_state(dispenser, hatch, pe_state).await,
        }
    }
}

// ---- State Handling Functions ----

async fn handle_running_state(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    hatch: &mut Hatch,
    photo_eye: &DigitalInput,
    database: &Data,
    bowl_count: &mut i64,
) {
    if let Some(snack) = get_snack(state.clone()) {
        let setpoint = Setpoint::Weight(WeightedDispense {
            setpoint: snack.min_setpoint as f64,
            timeout: Duration::from_micros(1000),
        });

        dispenser
            .launch_dispense(setpoint, Parameters::from(snack.dispense_parameters.clone()))
            .await;

        handle_user_selection(state.clone(), dispenser, &snack).await;

        wait_for_pe(photo_eye).await;
        hatch.open().await.unwrap();
        sleep(Duration::from_millis(1000)).await;
        hatch.close().await.unwrap();

        // Update bowl count
        *bowl_count += 1;
        update_ui_bowl_count(state.clone(), *bowl_count);

        database.log(DataAction::Dispensed, Some(snack.id)).unwrap();
        reset_ui_request(state.clone());
    }
}

async fn handle_user_selection(
    state: tauri::State<'_, Mutex<AppData>>,
    dispenser: &DispenseHandle,
    snack: &Ingredient,
) {
    loop {
        match get_ui_request(state.clone()) {
            UiRequest::None => sleep(Duration::from_millis(250)).await,
            UiRequest::SmallDispense => break,
            UiRequest::RegularDispense => {
                let sp = snack.max_setpoint - snack.min_setpoint;
                let setpoint = Setpoint::Weight(WeightedDispense {
                    setpoint: sp as f64,
                    timeout: Duration::from_micros(1000),
                });
                dispenser
                    .launch_dispense(setpoint, Parameters::from(snack.dispense_parameters.clone()))
                    .await;
                break;
            }
        }
    }
}

async fn handle_emptying_state(dispenser: &DispenseHandle, hatch: &mut Hatch, pe_state: PhotoEyeState) {
    hatch.open().await.unwrap();
    if matches!(pe_state, PhotoEyeState::Blocked) {
        dispenser.empty().await;
    } else {
        dispenser.disable().await;
    }
}

