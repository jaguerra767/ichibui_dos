use control_components::components::clear_core_io::DigitalInput;
use control_components::components::clear_core_motor::ClearCoreMotor;
use control_components::controllers::clear_core::{Controller, MotorBuilder};
use rusqlite::Connection;

use crate::config::Config;
use crate::data_logging::Data;
use crate::hatch::Hatch;
use crate::HOME_DIRECTORY;

const DB_PATH: &str = ".config/ichibu/data/";

#[derive(Debug, Default, Clone)]

pub enum PhotoEyeState {
    Blocked,
    #[default]
    Unblocked,
}

pub async fn photo_eye_state(input: &DigitalInput) -> PhotoEyeState {
    if input.get_state().await {
        PhotoEyeState::Blocked
    } else {
        PhotoEyeState::Unblocked
    }
}

pub async fn setup_conveyor_motor(config: &Config, controller: &Controller) -> ClearCoreMotor {
    let motor_id = config.motor.id;
    let motor = controller.get_motor(motor_id);
    motor.clear_alerts().await;
    motor.set_acceleration(config.motor.acceleration).await;
    motor.set_deceleration(config.motor.acceleration).await;
    motor
}

pub fn initialize_database() -> (Data, i64) {
    let database_path = format!("{}/{}", HOME_DIRECTORY.as_str(), DB_PATH);
    let database_connection = Connection::open(database_path).unwrap();
    let database = Data::new(database_connection);
    let bowl_count = database.connect().unwrap();
    (database, bowl_count)
}

pub fn initialize_controller(config: &Config) -> Controller {
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
    tauri::async_runtime::spawn(async move {
        if let Err(_) = controller_client.await {
            log::warn!("No motor/io controller connected, running in demo mode");
        }
    });
    controller
}

pub async fn initialize_hatch(cc_handle: &Controller, config: &Config) -> Hatch {
    let mut hatch = Hatch::new(
        cc_handle.get_motor(config.hatch.motor_id),
        cc_handle.get_digital_input(config.hatch.open_input),
        cc_handle.get_digital_input(config.hatch.close_input),
    );
    hatch.setup().await;
    hatch
}
