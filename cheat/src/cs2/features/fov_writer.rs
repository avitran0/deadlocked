use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    thread::{self, Builder, JoinHandle},
    time::Duration,
};

use crate::{
    cs2::offsets::Offsets,
    os::process::Process,
};

// ~10k iterations/sec: far above the engine's per-frame ramp update,
// so the pinned state is re-asserted many times within a single rendered frame.
const ACTIVE_INTERVAL: Duration = Duration::from_micros(100);
const INACTIVE_INTERVAL: Duration = Duration::from_millis(10);

pub struct FovWriter {
    slot: Arc<Mutex<Option<(Process, Offsets)>>> ,
    generation: Arc<AtomicU64>,
    active: Arc<AtomicBool>,
    value: Arc<AtomicU32>,
    _thread: Option<JoinHandle<()>>,
}

impl FovWriter {
    pub fn new() -> Self {
        let slot: Arc<Mutex<Option<(Process, Offsets)>>> = Arc::new(Mutex::new(None));
        let generation = Arc::new(AtomicU64::new(0));
        let active = Arc::new(AtomicBool::new(false));
        let value = Arc::new(AtomicU32::new(0));

        let thread = Builder::new()
            .name("fov-writer".to_string())
            .spawn({
                let slot = slot.clone();
                let generation = generation.clone();
                let active = active.clone();
                let value = value.clone();
                move || Self::run(slot, generation, active, value)
            })
            .ok();

        Self {
            slot,
            generation,
            active,
            value,
            _thread: thread,
        }
    }

    pub fn publish(&self, process: Process, offsets: Offsets) {
        *self.slot.lock().unwrap() = Some((process, offsets));
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set(&self, active: bool, value: u32) {
        self.value.store(value, Ordering::Relaxed);
        self.active.store(active, Ordering::Relaxed);
    }

    fn run(
        slot: Arc<Mutex<Option<(Process, Offsets)>>> ,
        generation: Arc<AtomicU64>,
        active: Arc<AtomicBool>,
        value: Arc<AtomicU32>,
    ) {
        // Cache the snapshot locally; re-pull only when the slot is republished.
        let mut cached: Option<(Process, Offsets)> = None;
        let mut last_generation: u64 = 0;

        loop {
            let generation = generation.load(Ordering::Relaxed);
            if generation != last_generation {
                last_generation = generation;
                cached = slot.lock().unwrap().clone();
            }

            if active.load(Ordering::Relaxed) {
                if let Some((process, offsets)) = &cached {
                    Self::enforce(&process, &offsets, value.load(Ordering::Relaxed));
                }
                thread::sleep(ACTIVE_INTERVAL);
            } else {
                thread::sleep(INACTIVE_INTERVAL);
            }
        }
    }

    fn local_player(process: &Process, offsets: &Offsets) -> Option<(usize, usize)> {
        let controller: usize = process.read(offsets.direct.local_player);
        if controller == 0 {
            return None;
        }
        let pawn_handle: i32 = process.read(controller + offsets.controller.pawn);
        if pawn_handle == -1 {
            return None;
        }
        let index = pawn_handle as usize & 0x7FFF;
        let bucket_index = index >> 9;
        let index_in_bucket = index & 0x1FF;
        let bucket_ptr: usize = process.read(offsets.interface.entity + 8 * bucket_index);
        if bucket_ptr == 0 {
            return None;
        }
        let pawn: usize = process.read(bucket_ptr + offsets.entity_identity.size * index_in_bucket);
        if pawn == 0 {
            return None;
        }
        Some((controller, pawn))
    }

    fn enforce(process: &Process, offsets: &Offsets, value: u32) {
        let Some((controller, pawn)) = Self::local_player(process, offsets) else {
            return;
        };

        // leave scoped zoom FOV untouched
        if process.read::<u8>(pawn + offsets.pawn.is_scoped) != 0 {
            return;
        }

        let camera: usize = process.read(pawn + offsets.pawn.camera_services);
        if camera == 0 {
            return;
        }

        let cam = &offsets.camera_services;
        process.write(camera + cam.fov, value);
        process.write(camera + cam.fov_start, value);
        process.write(camera + cam.fov_time, 0.0_f32);
        process.write(camera + cam.fov_rate, 0.0_f32);
        process.write(camera + cam.zoom_owner, 0xFFFF_FFFF_u32);
        process.write(camera + cam.last_shot_fov, value as f32);
        process.write(controller + offsets.controller.desired_fov, value);
    }
}
