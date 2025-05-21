use crate::config::Config;

use super::CS2;

impl CS2 {
    pub fn esp(&self, config: &Config) {
        // 0xC3 is RET, 0x48 is TEST
        let instruction: u8 = self.process.read(self.offsets.direct.is_other_enemy);

        if config.misc.esp && instruction != 0xC3 {
            self.process
                .write_file::<u8>(self.offsets.direct.is_other_enemy, 0xC3);
        } else if !config.misc.esp && instruction != 0x48 {
            self.process
                .write_file::<u8>(self.offsets.direct.is_other_enemy, 0x48);
        }
    }
}
