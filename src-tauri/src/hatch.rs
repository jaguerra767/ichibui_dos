use crate::config::HatchConfig;
use async_clear_core::io::DigitalInput;
use async_clear_core::motor::ClearCoreMotor;
use std::{fmt, time::Duration};
use tokio::time::{interval, Instant};

// TODO: maybe put these in config as well?
pub const HATCH_TIMEOUT: Duration = Duration::from_secs(6);
pub const HATCH_STROKE: f64 = 100_000.;
#[derive(Debug)]
pub enum HatchError {
    Timeout,
}

impl fmt::Display for HatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self )
    }
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
    pub async fn setup(&mut self, config: &HatchConfig) -> anyhow::Result<()> {
        self.motor.enable().await?;
        self.motor.clear_alerts().await?;
        self.motor.set_velocity(config.velocity).await?;
        self.motor.set_acceleration(config.acceleration).await?;
        self.motor.set_deceleration(config.acceleration).await?;
        Ok(())
    }
    pub async fn open(&mut self) -> anyhow::Result<()> {
        if self
            .open_input
            .get_state()
            .await
            .expect("Unable to get state from sensor")
        {
            return Ok(());
        }

        let start_time = Instant::now();
        let mut interval = interval(Duration::from_millis(100));
        self.motor.relative_move(-HATCH_STROKE).await.unwrap();
        while !self
            .open_input
            .get_state()
            .await
            .expect("Unable to get state from sensor")
        {
            if Instant::now() - start_time > HATCH_TIMEOUT {
                self.motor.abrupt_stop().await?;
                return Err(anyhow::anyhow!(HatchError::Timeout));
            }
            interval.tick().await;
        }
        self.motor.abrupt_stop().await?;
        Ok(())
    }
    pub async fn close(&mut self) -> anyhow::Result<()> {
        if self
            .close_input
            .get_state()
            .await
            .expect("Unable to get state from sensor")
        {
            return Ok(());
        }
        let start_time = Instant::now();
        let mut interval = interval(Duration::from_millis(100));
        let _ = self.motor.relative_move(HATCH_STROKE).await;
        while !self
            .close_input
            .get_state()
            .await
            .expect("Unable to get state from sensor")
        {
            if Instant::now() - start_time > HATCH_TIMEOUT {
                self.motor.abrupt_stop().await?;
                self.motor.relative_move(-HATCH_STROKE).await.unwrap();
                return Err(anyhow::anyhow!(HatchError::Timeout));
            }
            interval.tick().await;
        }
        self.motor.abrupt_stop().await?;
        Ok(())
    }
}
