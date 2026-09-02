use crate::cs2::{CS2, offsets::Offsets};

impl CS2 {
    pub fn find_offsets(&self) -> Option<Offsets> {
        let proc = &self.process;
        let mut offsets = Offsets::default();
        find_offsets_body!(proc, offsets, schema, client, physics);
        Some(offsets)
    }
}
