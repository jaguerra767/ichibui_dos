use control_components::components::clear_core_io::DigitalInput;
use control_components::components::clear_core_motor::ClearCoreMotor;
use std::time::Duration;
use tokio::time::{interval, Instant};

pub const HATCH_TIMEOUT: Duration = Duration::from_secs(6);
pub const HATCH_STROKE: f64 = 350.;
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
    pub async fn setup(&mut self) {
        self.motor.enable().await.unwrap();
        self.motor.clear_alerts().await;
        self.motor.set_velocity(50.).await;
        self.motor.set_acceleration(250.).await;
        self.motor.set_deceleration(250.).await;
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
        self.motor.relative_move(HATCH_STROKE).await.unwrap();
        while !self.close_input.get_state().await {
            if Instant::now() - start_time > HATCH_TIMEOUT {
                self.motor.abrupt_stop().await;
                self.motor.relative_move(-HATCH_STROKE).await.unwrap();
                return Err(HatchError::Timeout);
            }
            interval.tick().await;
        }
        self.motor.abrupt_stop().await;
        Ok(())
    }
}

//
// #[tokio::test]
// async fn hatch_cycle() {
//     let mut io_set = JoinSet::new();
//     let (cc, cc_cl) = CCController::with_client(
//         "192.168.1.12:8888",
//         &[MotorBuilder {
//             id: 0,
//             scale: 800,
//         }],
//     );
//     io_set.spawn(cc_cl);
//     info!("Connected!");
//     let mut hatch = Hatch::new(
//         cc.get_output(0),
//         cc.get_output(1),
//         Duration::from_millis(1700),
//         cc.get_analog_input(3),
//         28,
//         10,
//         Duration::from_secs(3)
//     );
//
//     hatch.open().await;
//     sleep(Duration::from_secs(3)).await;
//     hatch.close().await;
// }
//
// #[tokio::test]
// async fn hatch_open() {
//     let mut io_set = JoinSet::new();
//     let (cc, cc_cl) = CCController::with_client(
//         "192.168.1.12:8888",
//         &[MotorBuilder {
//             id: 0,
//             scale: 800,
//         }],
//     );
//     io_set.spawn(cc_cl);
//     info!("Connected!");
//     let mut hatch = Hatch::new(
//         cc.get_output(0),
//         cc.get_output(1),
//         Duration::from_millis(1700),
//         cc.get_analog_input(3),
//         28,
//         10,
//         Duration::from_secs(3)
//     );
//
//     hatch.open().await;
// }
//
// #[tokio::test]
// async fn hatch_close() {
//     let mut io_set = JoinSet::new();
//     let (cc, cc_cl) = CCController::with_client(
//         "192.168.1.12:8888",
//         &[MotorBuilder {
//             id: 0,
//             scale: 800,
//         }],
//     );
//     io_set.spawn(cc_cl);
//     info!("Connected!");
//     let mut hatch = Hatch::new(
//         cc.get_output(0),
//         cc.get_output(1),
//         Duration::from_millis(1700),
//         cc.get_analog_input(3),
//         28,
//         10,
//         Duration::from_secs(3)
//     );
//
//     hatch.close().await;
// }
//
// #[tokio::test]
// async fn diagnose() {
//     env_logger::init();
//     let mut io_set = JoinSet::new();
//     let (cc, cc_cl) = CCController::with_client(
//         "192.168.1.12:8888",
//         &[MotorBuilder {
//             id: 0,
//             scale: 800,
//         }],
//     );
//     io_set.spawn(cc_cl);
//     info!("Connected!");
//     let mut hatch = Hatch::new(
//         cc.get_output(0),
//         cc.get_output(1),
//         Duration::from_millis(1700),
//         cc.get_analog_input(3),
//         28,
//         10,
//         Duration::from_secs(3)
//     );
//
//     let (open_data, close_data) = hatch.diagnose().await;
//     info!("Open: {:?}", open_data);
//     info!("Close: {:?}", close_data);
//
// }
