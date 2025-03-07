// use crate::ichibu::Ichibu;
use control_components::components::clear_core_io::DigitalInput;
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

// pub struct PhotoEye {
//     input: DigitalInput,
// }
// impl PhotoEye {
//     pub fn new(input: DigitalInput) -> Self {
//         Self { input }
//     }
//     pub async fn check(&self) -> PhotoEyeState {
//         if self.input.get_state().await {
//             PhotoEyeState::Blocked
//         } else {
//             PhotoEyeState::Unblocked
//         }
//     }

//     fn compare_states(state: PhotoEyeState, others_state: PhotoEyeState) -> bool {
//         matches!(
//             (state, others_state),
//             (PhotoEyeState::Unblocked, PhotoEyeState::Unblocked)
//                 | (PhotoEyeState::Blocked, PhotoEyeState::Blocked)
//         )
//     }
//     pub async fn wait_for_state(
//         &self,
//         target_state: PhotoEyeState,
//         ichibu: &mut Ichibu,
//     ) -> Result<(), ()> {
//         let mut pe_state = self.check().await;
//         let mut interval = interval(CONFIG.get().unwrap().photo_eye.sample_period);
//         loop {
//            // ichibu.update_from_ui().await;
//             if ichibu.is_in_idle_state().await {
//                 return Err(());
//             }
//             if PhotoEye::compare_states(pe_state, target_state) {
//                 for i in 0..CONFIG.get().unwrap().photo_eye.sample_number {
//                     if !PhotoEye::compare_states(pe_state, target_state) {
//                         break;
//                     }
//                     if i == CONFIG.get().unwrap().photo_eye.sample_number - 1 {
//                         return Ok(());
//                     }
//                     pe_state = self.check().await;
//                     interval.tick().await;
//                 }
//             }
//             pe_state = self.check().await;
//             interval.tick().await;
//         }
//     }
// }

// #[tokio::test]
// async fn compare() {
//     assert!(PhotoEye::compare_states(
//         PhotoEyeState::Unblocked,
//         PhotoEyeState::Unblocked
//     ));
//     assert!(PhotoEye::compare_states(
//         PhotoEyeState::Blocked,
//         PhotoEyeState::Blocked
//     ));
//     assert!(!PhotoEye::compare_states(
//         PhotoEyeState::Blocked,
//         PhotoEyeState::Unblocked
//     ));
// }
