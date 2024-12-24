use constants::Constants;
use glam::Vec2;
use log::info;
use offsets::Offsets;
use target::Target;

use crate::{
    aimbot::Aimbot,
    proc::{get_pid_proton, open_process, validate_pid},
    process::Process,
};

mod constants;
mod offsets;
mod target;

#[derive(Debug)]
pub struct Deadlock {
    is_valid: bool,
    process: Option<Process>,
    offsets: Offsets,
    target: Target,

    previous_aim_punch: Vec2,
    unaccounted_aim_punch: Vec2,
}

impl Aimbot for Deadlock {
    fn is_valid(&self) -> bool {
        if let Some(process) = &self.process {
            return self.is_valid && validate_pid(process.pid);
        }
        false
    }

    fn setup(&mut self) {
        let pid = match get_pid_proton(Constants::PROCESS_NAME) {
            Some(pid) => pid,
            None => {
                self.is_valid = false;
                return;
            }
        };

        let process = match open_process(pid) {
            Some(process) => process,
            None => {
                self.is_valid = false;
                return;
            }
        };
        info!("game started, pid: {}", pid);

        /*self.offsets = match self.find_offsets(&process) {
            Some(offsets) => offsets,
            None => {
                self.is_valid = false;
                return;
            }
        };
        info!("offsets found");

        self.process = Some(process);
        self.is_valid = true;*/
    }

    fn run(&mut self, config: &crate::config::Config, mouse: &mut std::fs::File) {
        todo!()
    }
}

impl Deadlock {
    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: None,
            offsets: Offsets::default(),
            target: Target::default(),

            previous_aim_punch: Vec2::ZERO,
            unaccounted_aim_punch: Vec2::ZERO,
        }
    }
}
