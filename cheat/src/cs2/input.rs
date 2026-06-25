use utils::DynamicBitSet;

use crate::{
    cs2::{key_codes::KeyCode, offsets::Offsets},
    os::process::Process,
};

#[derive(Debug)]
pub struct Input {
    previous_state: DynamicBitSet,
    current_state: DynamicBitSet,
}

impl Input {
    const MAX_KEY: u64 = 512;

    pub fn new() -> Self {
        Self {
            previous_state: DynamicBitSet::new(),
            current_state: DynamicBitSet::new(),
        }
    }

    pub fn update(&mut self, process: &Process, offsets: &Offsets) {
        let state = process.read_bytes(
            offsets.interface.input + offsets.direct.button_state,
            Self::MAX_KEY / 8,
        );

        std::mem::swap(&mut self.previous_state, &mut self.current_state);
        self.current_state = DynamicBitSet::from(state);
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        if key == KeyCode::None {
            return false;
        }
        self.current_state.get(key.usize()).unwrap_or(false)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        if key == KeyCode::None {
            return false;
        }
        !self.previous_state.get(key.usize()).unwrap_or(false)
            && self.current_state.get(key.usize()).unwrap_or(false)
    }
}
