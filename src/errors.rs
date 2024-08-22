use thiserror::Error;

use mimic_emulator::errors::MimicError;

#[derive(Error, Debug)]
pub enum SuperMipsError {
    // #[error("{}", .0)]
    #[error(transparent)]
    MimicError(#[from] MimicError),
}

// impl From<MimicError> for SuperMipsError {
//     fn from(mimic_error: MimicError) -> Self {
//         Self::MimicError(mimic_error)
//     }
// }
