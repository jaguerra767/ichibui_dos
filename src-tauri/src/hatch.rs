use crate::config::HatchConfig;
use control_components::components::clear_core_io::DigitalInput;
use control_components::components::clear_core_motor::ClearCoreMotor;
use std::time::Duration;
use tokio::time::{interval, Instant};

// TODO: maybe put these in config as well?
pub const HATCH_TIMEOUT: Duration = Duration::from_secs(6);
pub const HATCH_STROKE: f64 = 100_000.;
#[derive(Debug)]
pub enum HatchError {
    Timeout,
}
pub struct Hatch {
    motor: ClearCoreMotor,
    open_input: DigitalInput,
    close_input: DigitalInput,
}
impl Hatch {
    pub fn new(motor: ClearCoreMotor, open_input: DigitalInput, close_input: DigitalInput) -> Self {
        Self {
            motor,
            open_input,
            close_input,
        }
    }
    pub async fn setup(&mut self, config: &HatchConfig) {
        self.motor.enable().await.unwrap();
        self.motor.clear_alerts().await;
        self.motor.set_velocity(config.velocity).await;
        self.motor.set_acceleration(config.acceleration).await;
        //self.motor.set_deceleration(config.acceleration).await;
    }
    pub async fn open(&mut self) -> Result<(), HatchError> {
        if self.open_input.get_state().await {
            return Ok(());
        }

        let start_time = Instant::now();
        let mut interval = interval(Duration::from_millis(100));
        self.motor.relative_move(-HATCH_STROKE).await.unwrap();
        while !self.open_input.get_state().await {
            if Instant::now() - start_time > HATCH_TIMEOUT {
                self.motor.abrupt_stop().await;
                return Err(HatchError::Timeout);
            }
            interval.tick().await;
        }
        self.motor.abrupt_stop().await;
        Ok(())
    }
    pub async fn close(&mut self) -> Result<(), HatchError> {
        if self.close_input.get_state().await {
            return Ok(());
        }
        let start_time = Instant::now();
        let mut interval = interval(Duration::from_millis(100));
        let _ = self.motor.relative_move(HATCH_STROKE).await;
        while !self.close_input.get_state().await {
            if Instant::now() - start_time > HATCH_TIMEOUT {
                self.motor.abrupt_stop().await;
                self.open().await?;
                return Err(HatchError::Timeout);
            }
            interval.tick().await;
        }
        self.motor.abrupt_stop().await;
        Ok(())
    }
}