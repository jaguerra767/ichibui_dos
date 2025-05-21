use crate::config::Config;
use crate::scale::{self, ScaleRequest};
use async_clear_core::motor::ClearCoreMotor;

use anyhow::{anyhow, Result};
use log::info;
use tokio::sync::mpsc;
use tokio::time::{interval, Instant, MissedTickBehavior};

pub struct Parameters {
    pub motor_speed: f64,
    pub min_speed: f64,
    pub check_offset: f64,
    pub sample_rate: f64,
    pub samples: usize,
    pub reverse_before: Option<f64>,
    pub reverse_after: Option<f64>,
}

const MOTOR_MOVE_POS: f64 = 20.0;

#[derive(Clone)]
pub struct DispenserIo {
    pub motor: ClearCoreMotor,
    pub scale: mpsc::Sender<ScaleRequest>,
}

async fn update_motor_speed(
    motor: ClearCoreMotor,
    error: f64,
    min_speed: f64,
    max_speed: f64,
) -> Result<()> {
    let speed = (error * max_speed).clamp(min_speed, max_speed);
    motor.set_velocity(speed).await?;
    motor.relative_move(MOTOR_MOVE_POS).await?;
    Ok(())
}

pub async fn dispense(io: DispenserIo, qty: f64, parameters: Parameters) -> Result<f64> {
    let config = tauri::async_runtime::spawn_blocking(Config::load).await?;
    let data_interval = config.phidget.data_interval;
    let dispense_timeout = config.dispense.timeout;
    let mut interval = interval(data_interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let motor = io.motor;
    let tx = io.scale;

    if let Some(rev) = parameters.reverse_before {
        motor.relative_move(-rev).await?;
        motor.wait_for_move(data_interval).await?
    }

    let start_time = Instant::now();
    let start_weight =
        scale::get_median_grams(tx.clone(), parameters.samples, data_interval).await?;
    let target_weight = start_weight - qty;

    let result = loop {
        let current_time = Instant::now();

        if (current_time - start_time) > dispense_timeout {
            motor.abrupt_stop().await?;
            return Err(anyhow!("Timed out!"));
        }

        let current_weight = scale::get_grams(tx.clone()).await?;
        if current_weight < target_weight + parameters.check_offset {
            motor.abrupt_stop().await?;
            motor.wait_for_move(data_interval).await?;

            info!("Checking weight");
            let current_weight =
                scale::get_median_grams(tx.clone(), parameters.samples, data_interval).await?;
            if current_weight < target_weight {
                break current_weight;
            } else {
                motor.relative_move(MOTOR_MOVE_POS).await?;
            }
        }
        let error = (current_weight - target_weight) / qty;
        update_motor_speed(
            motor.clone(),
            error,
            parameters.min_speed,
            parameters.motor_speed,
        )
        .await?;
        interval.tick().await;
    };

    if let Some(rev) = parameters.reverse_after {
        motor.relative_move(-rev).await?;
    }
    Ok(result)
}
