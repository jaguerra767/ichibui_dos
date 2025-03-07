use control_components::components::clear_core_io::DigitalInput;
use control_components::components::clear_core_motor::ClearCoreMotor;
use control_components::components::scale::{Scale, ScaleCmd};
use control_components::controllers::clear_core::{Controller, MotorBuilder};
use log::info;
use serde::de::Error;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;

use crate::config::Config;
use crate::hatch::Hatch;
// use tokio::time::interval;

#[derive(Debug, Default)]
pub enum PhotoEyeState {
    Blocked,
    #[default]
    Unblocked,
}

pub struct PhotoEye {
    input: DigitalInput,
}

impl PhotoEye {
    pub fn new(input: DigitalInput) -> Self {
        Self { input }
    }

    pub async fn get_state(&self) -> PhotoEyeState {
        if self.input.get_state().await {
            PhotoEyeState::Blocked
        } else {
            PhotoEyeState::Unblocked
        }
    }
}

pub async fn photo_eye_state(input: DigitalInput) -> PhotoEyeState {
    if input.get_state().await {
        PhotoEyeState::Blocked
    } else {
        PhotoEyeState::Unblocked
    }
}

pub struct IchibuIo {
    pub io_handle: JoinSet<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    pub controller: Controller,
    pub scale: Sender<ScaleCmd>,
}

pub async fn launch_io(config: &Config) -> IchibuIo {
    let mut io_set = JoinSet::new();

    info!("Connecting Phidget...");
    let mut scale = Scale::new(config.phidget.sn);
    scale = Scale::change_coefficients(scale, config.phidget.coefficients.to_vec());
    scale = scale.connect().unwrap();
    let (scale_tx, scale_actor) = scale.actor_tx_pair();
    io_set.spawn(scale_actor);
    info!("Connected!");

    info!("Connecting ClearCore...");
    let (cc, cc_cl) = Controller::with_client(
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
    io_set.spawn(cc_cl);
    IchibuIo {
        io_handle: io_set,
        controller: cc,
        scale: scale_tx,
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

pub async fn setup_hatch(config: &Config, controller: &Controller) -> Hatch {
    let mut hatch = Hatch::new(
        controller.get_motor(config.hatch.motor_id),
        controller.get_digital_input(config.hatch.open_input),
        controller.get_digital_input(config.hatch.close_input),
    );
    hatch.setup().await;
    hatch
}
