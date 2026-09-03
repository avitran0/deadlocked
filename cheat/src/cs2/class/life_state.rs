#[repr(u8)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LifeState {
    Alive = 0,
    Dying = 1,
    #[default]
    Dead = 2,
    Respawnable = 3,
    Respawning = 4,
}

impl LifeState {
    pub fn is_alive(self) -> bool {
        self == Self::Alive
    }
}

impl TryFrom<u8> for LifeState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Alive,
            1 => Self::Dying,
            2 => Self::Dead,
            3 => Self::Respawnable,
            4 => Self::Respawning,
            _ => return Err(()),
        })
    }
}
