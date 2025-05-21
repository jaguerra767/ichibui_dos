use async_clear_core::controller::ControllerHandle;
use async_clear_core::io::DigitalInput;
use async_clear_core::motor::ClearCoreMotor;
use async_clear_core::motor::MotorBuilder;

use anyhow::Result;
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

pub async fn photo_eye_state(input: &DigitalInput) -> Result<PhotoEyeState> {
    if input.get_state().await? {
        Ok(PhotoEyeState::Blocked)
    } else {
        Ok(PhotoEyeState::Unblocked)
    }
}

pub async fn setup_conveyor_motor(
    config: &Config,
    controller: ControllerHandle,
) -> Result<ClearCoreMotor> {
    let motor_id = config.motor.id;
    let motor = controller.get_motor(motor_id);
    motor.clear_alerts().await?;
    motor.set_acceleration(config.motor.acceleration).await?;
    motor.set_deceleration(config.motor.acceleration).await?;
    Ok(motor)
}

pub fn initialize_database() -> (Data, i64) {
    let database_path = format!("{}/{}", HOME_DIRECTORY.as_str(), DB_PATH);
    let database_connection = Connection::open(database_path).unwrap();
    let database = Data::new(database_connection);
    let bowl_count = database.connect().unwrap();
    (database, bowl_count)
}

pub fn initialize_controller(config: &Config) -> ControllerHandle {
    ControllerHandle::new(
        config.addresses.clear_core.clone(),
        [
            MotorBuilder {
                id: config.motor.id,
                scale: config.motor.scale,
            },
            MotorBuilder {
                id: config.hatch.motor_id,
                scale: config.hatch.scale,
            },
            MotorBuilder {
                id: 2,
                scale: config.motor.scale,
            },
            MotorBuilder {
                id: 3,
                scale: config.hatch.scale,
            },
        ],
    )
}

pub async fn initialize_hatch(cc_handle: ControllerHandle, config: &Config) -> anyhow::Result<Hatch>{
    let mut hatch = Hatch::new(
        cc_handle.get_motor(config.hatch.motor_id),
        cc_handle.get_digital_input(config.hatch.open_input),
        cc_handle.get_digital_input(config.hatch.close_input),
    );
    hatch.setup(&config.hatch).await?;
    Ok(hatch)
}
