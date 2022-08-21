use serde::Deserialize;

use crate::units::Length;

#[derive(Deserialize)]
pub(super) struct Parameters {
    pub softening_length: Length,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            softening_length: Length::zero(),
        }
    }
}
