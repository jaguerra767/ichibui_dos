use std::time::Duration;

use control_components::{
    components::{clear_core_motor::ClearCoreMotor, scale::ScaleCmd},
    subsystems::dispenser::{Dispenser, Parameters, Setpoint},
};

use log::error;
use tokio::sync::{mpsc, oneshot};

struct Dispense {
    receiver: mpsc::Receiver<DispenseMsg>,
    scale_tx: mpsc::Sender<ScaleCmd>,
    motor: ClearCoreMotor,
    timeout: Duration,
}

enum DispenseMsg {
    LaunchDispense {
        setpoint: Setpoint,
        parameters: Parameters,
        respond_to: oneshot::Sender<()>,
    },
    Enable,
    Disable,
    Empty,
}

impl Dispense {
    fn new(
        receiver: mpsc::Receiver<DispenseMsg>,
        motor: ClearCoreMotor,
        scale_tx: mpsc::Sender<ScaleCmd>,
    ) -> Self {
        let default_timeout = Duration::from_secs(10);
        Self {
            receiver,
            scale_tx,
            motor,
            timeout: default_timeout,
        }
    }

    async fn handle_msg(&self, msg: DispenseMsg) {
        match msg {
            DispenseMsg::LaunchDispense {
                setpoint,
                parameters,
                respond_to,
            } => {
                let _ = self.motor.enable().await;
                Dispenser::new(
                    self.motor.clone(),
                    setpoint,
                    parameters,
                    self.scale_tx.clone(),
                )
                .dispense(self.timeout)
                .await;
                let _ = respond_to.send(());
            }
            DispenseMsg::Disable => {
                self.motor.abrupt_stop().await;
                self.motor.disable().await;
            }
            DispenseMsg::Empty => {
                let _ = self.motor.enable().await;
                tokio::time::sleep(Duration::from_millis(1000)).await;
                self.motor.clear_alerts().await;
                self.motor.set_velocity(1.).await;
                let _ = self.motor.relative_move(100.).await;
            }
            DispenseMsg::Enable => {
                if let Err(e) = self.motor.enable().await {
                    error!("Motor failed to enable{:?}", e)
                }
            }
        }
    }
}

async fn run_dispense_actor(mut dispense: Dispense) {
    while let Some(msg) = dispense.receiver.recv().await {
        dispense.handle_msg(msg).await;
    }
}

#[derive(Clone)]
pub struct DispenseHandle {
    sender: mpsc::Sender<DispenseMsg>,
}

impl DispenseHandle {
    pub fn new(motor: ClearCoreMotor, scale_tx: mpsc::Sender<ScaleCmd>) -> Self {
        let (sender, receiver) = mpsc::channel(10);
        let dispenser = Dispense::new(receiver, motor, scale_tx);
        tauri::async_runtime::spawn(run_dispense_actor(dispenser));
        Self { sender }
    }

    pub async fn launch_dispense(&self, setpoint: Setpoint, parameters: Parameters) {
        let (send, recv) = oneshot::channel();
        let msg = DispenseMsg::LaunchDispense {
            setpoint,
            parameters,
            respond_to: send,
        };
        let _ = self.sender.send(msg).await;
        recv.await.expect("Dispenser task has been killed")
    }

    pub async fn empty(&self) {
        let msg = DispenseMsg::Empty {};
        let _ = self.sender.send(msg).await;
    }

    pub async fn disable(&self) {
        let msg = DispenseMsg::Disable {};
        let _ = self.sender.send(msg).await;
    }

    pub async fn enable(&self) {
        let msg = DispenseMsg::Enable {};
        let _ = self.sender.send(msg).await;
    }
}
