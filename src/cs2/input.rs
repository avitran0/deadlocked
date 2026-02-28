use utils::bitset::BitSet;
use crate::{
    cs2::{key_codes::KeyCode, offsets::Offsets},
    os::process::Process,
};

#[derive(Debug)]
pub struct Input {
    previous_state: BitSet,
    current_state: BitSet,
}

impl Input {
    const MAX_KEY: usize = 512;

    pub fn new() -> Self {
        let bs = BitSet::with_capacity(Self::MAX_KEY);

        Self {
            previous_state: bs.clone(),
            current_state: bs,
        }
    }

    pub fn update(&mut self, process: &Process, offsets: &Offsets) {
        let bytes = process.read_bytes(
            offsets.interface.input + offsets.direct.button_state,
            (Self::MAX_KEY / 8) as u64,

        );

        self.previous_state = self.current_state.clone();

        self.current_state = BitSet::from_vec(bytes);
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        let idx = key.usize();
        if idx >= Self::MAX_KEY {
            return false;
        }
        self.current_state.get_checked(idx).unwrap_or(false)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        let idx = key.usize();
        if idx >= Self::MAX_KEY {
            return false;
        }
        let prev = self.previous_state.get_checked(idx).unwrap_or(false);
        let curr = self.current_state.get_checked(idx).unwrap_or(false);
        !prev && curr
    }
}
