use control_components::components::clear_core_io::{HBridge, HBridgeState};
use control_components::controllers::clear_core::Controller;

#[derive(Clone)]
pub struct Lights {
    red: HBridge,
    green: HBridge,
}
impl Lights {
    pub fn new(controller: Controller) -> Self {
        Self {
            red: controller.get_h_bridge(4),
            green: controller.get_h_bridge(5),
        }
    }
    pub async fn set_color(&mut self, color: LightColors) {
        match color {
            LightColors::Red => {
                self.red.set_state(HBridgeState::Neg).await;
                self.green.set_state(HBridgeState::Off).await;
            }
            LightColors::Green => {
                self.red.set_state(HBridgeState::Pos).await;
                self.green.set_state(HBridgeState::Neg).await;
            }
            LightColors::Yellow => {
                self.red.set_state(HBridgeState::Neg).await;
                self.green.set_state(HBridgeState::Neg).await;
            }
        }
    }
    pub async fn turn_off(&mut self) {
        self.red.set_state(HBridgeState::Pos).await;
        self.green.set_state(HBridgeState::Pos).await;
    }
}
pub enum LightColors {
    Red,
    Yellow,
    Green
}