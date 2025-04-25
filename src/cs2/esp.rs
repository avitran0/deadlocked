use crate::config::Config;

use super::CS2;

impl CS2 {
    pub fn esp(&self, config: &Config) {
        let process = match &self.process {
            Some(process) => process,
            None => return,
        };

        let instruction: u8 = process.read(self.offsets.direct.is_other_enemy);

        if !config.misc.esp {
            if instruction != 0x48 {
                process.write_file::<u8>(self.offsets.direct.is_other_enemy, 0x48);
            }
            return;
        }

        if instruction != 0xC3 {
            process.write_file::<u8>(self.offsets.direct.is_other_enemy, 0xC3);
        }
    }
}
