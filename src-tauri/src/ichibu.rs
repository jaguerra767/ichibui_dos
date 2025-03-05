use crate::config::Config;
use crate::data_logging::{Data, DataAction};
use control_components::components::scale::ScaleCmd;
use control_components::controllers::clear_core::Controller;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

type CCController = Controller;
#[derive(Debug, Clone, Deserialize)]
pub struct CustomIngredient {
    pub serving_size: f64,
    pub speed: f64,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum DispenseType {
    #[default]
    Classic,
    LargeSmall,
}

#[derive(Debug, Clone)]
pub enum UICmd {
    SetRunState(RunState),
    SetIngredientId(usize),
    SetCustomIngredient(CustomIngredient),
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum RunState {
    #[default]
    Ready,
    Running,
    Cleaning,
    Emptying,
}
#[derive(Default, Clone, Debug, Serialize)]
pub enum NodeLevel {
    #[default]
    Filled,
    Empty,
}

#[derive(Default, Debug, Serialize, Clone)]
pub struct State {
    pub run_state: RunState,
    pub dispense_type: DispenseType,
    pub hatch_state: HatchState,
    pub node_level: NodeLevel,
    pub ingredient_id: usize,
    pub bowl_count: usize,
}
impl State {
    pub fn new(config: &Config, bowl_count: usize, starting_weight: f64) -> Self {
        Self {
            run_state: RunState::Ready,
            hatch_state: HatchState::Empty,
            dispense_type: DispenseType::Classic,
            node_level: {
                if starting_weight > config.setpoint.empty {
                    NodeLevel::Filled
                } else {
                    NodeLevel::Empty
                }
            },
            ingredient_id: 0,
            bowl_count,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ichibu {
    state: Arc<Mutex<State>>,
    cmd: Arc<Mutex<Option<UICmd>>>,
    custom_ingredient: Option<CustomIngredient>,
}
impl Ichibu {
    pub fn new(state: Arc<Mutex<State>>, cmd: Arc<Mutex<Option<UICmd>>>) -> Self {
        Self {
            state,
            cmd,
            custom_ingredient: Some(CustomIngredient {
                serving_size: 0.,
                speed: 0.,
            }),
        }
    }
    pub async fn update_from_ui(&mut self) {
        let cmd = self.cmd.lock().await.clone();
        if let Some(new_cmd) = cmd {
            match new_cmd {
                UICmd::SetIngredientId(new_id) => self.update_ingredient_id(new_id).await,
                UICmd::SetRunState(new_state) => self.update_run_state(new_state).await,
                UICmd::SetCustomIngredient(custom_ingredient) => {
                    self.set_custom_ingredient(custom_ingredient)
                }
            }
        }
        let mut new_cmd = self.cmd.lock().await;
        *new_cmd = None;
    }
    pub async fn get_run_state(&self) -> RunState {
        self.state.lock().await.run_state.clone()
    }
    pub async fn get_ingredient_id(&self) -> usize {
        self.state.lock().await.ingredient_id.clone()
    }
    pub async fn get_hatch_state(&self) -> HatchState {
        self.state.lock().await.hatch_state.clone()
    }
    pub async fn fill_hatch(&mut self) {
        let mut state = self.state.lock().await;
        if state.hatch_state.is_empty() {
            state.hatch_state = HatchState::Filled
        } else {
            warn!("Hatch was not empty, cannot fill")
        }
    }
    pub async fn empty_hatch(&mut self) {
        let mut state = self.state.lock().await;
        if state.hatch_state.is_filled() {
            state.hatch_state = HatchState::Empty
        } else {
            warn!("Hatch was not full, cannot empty")
        }
    }

    pub async fn update_ingredient_id(&mut self, new_id: usize) {
        info!("Ingredient ID updated to {:?}", new_id);
        let mut state = self.state.lock().await;
        state.ingredient_id = new_id;
    }
    fn set_custom_ingredient(&mut self, custom_ingredient: CustomIngredient) {
        info!("New custom ingredient: {:?}", custom_ingredient);
        self.custom_ingredient = Some(custom_ingredient);
    }
    pub fn get_custom_ingredient(&self) -> Option<CustomIngredient> {
        self.custom_ingredient.clone()
    }
    pub async fn update_run_state(&mut self, new_state: RunState) {
        let mut state = self.state.lock().await;
        match (&state.run_state, &new_state) {
            (RunState::Ready, RunState::Running) => {
                info!("Starting cycle");
                state.run_state = new_state;
            }
            (RunState::Running, RunState::Ready) => {
                info!("Stopping cycle");
                state.run_state = new_state;
            }
            (RunState::Ready, RunState::Cleaning) => {
                info!("Entering cleaning mode");
                state.run_state = new_state;
            }
            (RunState::Running, RunState::Cleaning) => {
                info!("Entering cleaning mode");
                state.run_state = new_state;
            }
            (RunState::Cleaning, RunState::Emptying) => {
                info!("Emptying");
                state.run_state = new_state;
            }
            (RunState::Emptying, RunState::Cleaning) => {
                info!("Returning to cleaning mode");
                state.run_state = new_state;
            }
            (RunState::Cleaning, RunState::Ready) => {
                info!("Returning to Ready");
                state.run_state = new_state;
            }
            (_, _) => {
                warn!("State request not possible");
                warn!("Tried {:?} from {:?}", state.run_state, new_state)
            }
        }
    }
    pub async fn is_in_idle_state(&self) -> bool {
        match self.state.lock().await.run_state {
            RunState::Running | RunState::Emptying => false,
            RunState::Ready | RunState::Cleaning => true,
        }
    }

    pub async fn get_bowl_count(&self) -> usize {
        self.state.lock().await.bowl_count
    }
    pub async fn update_bowl_count(&mut self) {
        let mut state = self.state.lock().await;
        let count = state.bowl_count;
        state.bowl_count = count + 1;
    }

    pub async fn get_node_level(&self) -> NodeLevel {
        let state = self.state.lock().await;
        state.node_level.clone()
    }
    pub async fn update_node_level(&mut self, config: &Config, io: &IchibuIo, database: &Data) {
        let samples = 50;
        // TODO: this is at 100Hz so takes half a second
        let mut weights_vec = Vec::with_capacity(50);
        for _i in 0..samples {
            let (weight_tx, weight_rx) = tokio::sync::oneshot::channel();
            io.get_scale_tx().send(ScaleCmd(weight_tx)).await.unwrap();
            weights_vec.push(weight_rx.await.unwrap());
        }
        weights_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut state = self.state.lock().await;
        let old_level = &state.node_level;

        let current_weight = weights_vec[samples / 2];
        debug!("Current Weight: {:}", current_weight);
        state.node_level = match old_level {
            NodeLevel::Filled => {
                if current_weight > config.setpoint.empty {
                    NodeLevel::Filled
                } else {
                    database.log(DataAction::RanOut, None).unwrap();
                    NodeLevel::Empty
                }
            }
            NodeLevel::Empty => {
                if current_weight > config.setpoint.empty + config.setpoint.filling_threshold {
                    database.log(DataAction::Refilled, None).unwrap();
                    NodeLevel::Filled
                } else {
                    NodeLevel::Empty
                }
            }
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize)]
pub enum HatchState {
    Filled,
    #[default]
    Empty,
}
impl HatchState {
    pub fn is_empty(&self) -> bool {
        matches!(self, HatchState::Empty)
    }
    pub fn is_filled(&self) -> bool {
        matches!(self, HatchState::Filled)
    }
}

#[derive(Clone)]
pub struct IchibuIo {
    cc: CCController,
    scale_tx: Sender<ScaleCmd>,
}
impl IchibuIo {
    pub fn new(cc: CCController, scale_tx: Sender<ScaleCmd>) -> Self {
        Self { cc, scale_tx }
    }
    pub fn get_controller(&self) -> CCController {
        self.cc.clone()
    }
    pub fn get_scale_tx(&self) -> Sender<ScaleCmd> {
        self.scale_tx.clone()
    }
}

pub enum Mode {
    Prod,
    Dev,
    Weigh,
}
