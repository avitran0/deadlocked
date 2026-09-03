use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Team {
    #[default]
    Unassigned = 0,
    Spectator = 1,
    T = 2,
    CT = 3,
}

impl Team {
    pub fn is_playing(self) -> bool {
        matches!(self, Self::T | Self::CT)
    }
}

impl TryFrom<u8> for Team {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Unassigned,
            1 => Self::Spectator,
            2 => Self::T,
            3 => Self::CT,
            _ => return Err(()),
        })
    }
}
