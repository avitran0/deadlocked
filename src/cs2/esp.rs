use crate::config::Config;

use super::CS2;

impl CS2 {
    pub fn esp(&self, config: &Config) {
        let instruction: u8 = self.process.read(self.offsets.direct.is_other_enemy);

        if !config.misc.esp {
            if instruction != 0x48 {
                self.process
                    .write_file::<u8>(self.offsets.direct.is_other_enemy, 0x48);
            }
            return;
        }

        if instruction != 0xC3 {
            self.process
                .write_file::<u8>(self.offsets.direct.is_other_enemy, 0xC3);
        }
    }
}
