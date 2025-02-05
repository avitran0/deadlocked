use std::{
    fs::{self, read_dir, File, OpenOptions},
    io::Write,
    os::unix::fs::FileTypeExt,
    time::{SystemTime, UNIX_EPOCH},
};

use glam::{IVec2, Vec2};
use log::warn;

use crate::key_codes::KeyCode;

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceStatus {
    Working(String),
    Disconnected,
    PermissionsRequired,
    NotFound,
}

#[derive(Debug, Clone, Copy)]
struct Timeval {
    seconds: u64,
    microseconds: u64,
}

#[derive(Debug, Clone, Copy)]
struct InputEvent {
    time: Timeval,
    event_type: u16,
    code: u16,
    value: i32,
}

impl InputEvent {
    fn bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(24);

        bytes.extend(&self.time.seconds.to_le_bytes());
        bytes.extend(&self.time.microseconds.to_le_bytes());

        bytes.extend(&self.event_type.to_le_bytes());
        bytes.extend(&self.code.to_le_bytes());
        bytes.extend(&self.value.to_le_bytes());

        bytes
    }
}

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const SYN_REPORT: u16 = 0x00;
const AXIS_X: u16 = 0x00;
const AXIS_Y: u16 = 0x01;
const BTN_LEFT: u16 = 0x110;
const KEY_SPACE: u16 = 57;

pub fn open_mouse() -> (File, DeviceStatus) {
    for file in read_dir("/dev/input").unwrap() {
        let entry = file.unwrap();
        if !entry.file_type().unwrap().is_char_device() {
            continue;
        }
        let name = entry.file_name().into_string().unwrap();
        if !name.starts_with("event") {
            continue;
        }
        // get device info from /sys/class/input
        let rel_bits: Vec<u64> =
            fs::read_to_string(format!("/sys/class/input/{}/device/capabilities/rel", name))
                .unwrap()
                .split_whitespace() // Handle multiple hex numbers
                .filter_map(|hex| u64::from_str_radix(hex, 16).ok()) // Decompose into individual bits
                .collect();
        let mut rel = Vec::new();
        for (i, hex) in rel_bits.iter().enumerate() {
            let bits = decompose_bits(*hex, i);
            rel.extend(bits);
        }

        if !rel.contains(&(AXIS_X as u64)) || !rel.contains(&(AXIS_Y as u64)) {
            continue;
        }

        let device_name =
            fs::read_to_string(format!("/sys/class/input/{}/device/name", name)).unwrap();

        let path = format!("/dev/input/{}", name);
        let file = OpenOptions::new().write(true).open(path);
        match file {
            Ok(file) => return (file, DeviceStatus::Working(device_name)),
            Err(_) => {
                warn!("please add your user to the input group or execute with sudo");
                warn!("without this, mouse movements will be written to /dev/null and discarded");
                let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
                return (file, DeviceStatus::PermissionsRequired);
            }
        }
    }

    let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
    warn!("no mouse found");
    (file, DeviceStatus::NotFound)
}

#[allow(unused)]
pub fn open_keyboard() -> (File, DeviceStatus) {
    for file in read_dir("/dev/input").unwrap() {
        let entry = file.unwrap();
        if !entry.file_type().unwrap().is_char_device() {
            continue;
        }
        let name = entry.file_name().into_string().unwrap();
        if !name.starts_with("event") {
            continue;
        }
        // get device info from /sys/class/input
        let key_bits: Vec<u64> =
            fs::read_to_string(format!("/sys/class/input/{}/device/capabilities/key", name))
                .unwrap()
                .split_whitespace()
                .filter_map(|hex| u64::from_str_radix(hex, 16).ok())
                .collect();
        let mut key = Vec::new();
        for (i, hex) in key_bits.iter().rev().enumerate() {
            let bits = decompose_bits(*hex, i);
            key.extend(bits);
        }

        if !key.contains(&(KEY_SPACE as u64)) {
            continue;
        }

        let device_name =
            fs::read_to_string(format!("/sys/class/input/{}/device/name", name)).unwrap();

        let path = format!("/dev/input/{}", name);
        let file = OpenOptions::new().write(true).open(path);
        match file {
            Ok(file) => return (file, DeviceStatus::Working(device_name)),
            Err(_) => {
                let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
                return (file, DeviceStatus::PermissionsRequired);
            }
        }
    }

    let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
    warn!("no keyboard found");
    (file, DeviceStatus::NotFound)
}

pub fn mouse_move(mouse: &mut File, coords: Vec2) {
    let coords = IVec2::new(coords.x as i32, coords.y as i32);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let time = Timeval {
        seconds: now.as_secs(),
        microseconds: now.subsec_micros() as u64,
    };

    let x = InputEvent {
        time,
        event_type: EV_REL,
        code: AXIS_X,
        value: coords.x,
    };

    let y = InputEvent {
        time,
        event_type: EV_REL,
        code: AXIS_Y,
        value: coords.y,
    };

    let syn = InputEvent {
        time,
        event_type: EV_SYN,
        code: SYN_REPORT,
        value: 0,
    };

    mouse.write_all(&x.bytes()).unwrap();
    mouse.write_all(&syn.bytes()).unwrap();

    mouse.write_all(&y.bytes()).unwrap();
    mouse.write_all(&syn.bytes()).unwrap();
}

pub fn mouse_left_press(mouse: &mut File) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let time = Timeval {
        seconds: now.as_secs(),
        microseconds: now.subsec_micros() as u64,
    };

    let key = InputEvent {
        time,
        event_type: EV_KEY,
        code: BTN_LEFT,
        value: 1,
    };

    let syn = InputEvent {
        time,
        event_type: EV_SYN,
        code: SYN_REPORT,
        value: 0,
    };

    mouse.write_all(&key.bytes()).unwrap();
    mouse.write_all(&syn.bytes()).unwrap();
}

pub fn mouse_left_release(mouse: &mut File) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let time = Timeval {
        seconds: now.as_secs(),
        microseconds: now.subsec_micros() as u64,
    };

    let key = InputEvent {
        time,
        event_type: EV_KEY,
        code: 0x110,
        value: 0,
    };

    let syn = InputEvent {
        time,
        event_type: EV_SYN,
        code: SYN_REPORT,
        value: 0,
    };

    mouse.write_all(&key.bytes()).unwrap();
    mouse.write_all(&syn.bytes()).unwrap();
}

#[allow(unused)]
pub fn keyboard_tap_key(keyboard: &mut File, key: KeyCode) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let time = Timeval {
        seconds: now.as_secs(),
        microseconds: now.subsec_micros() as u64,
    };

    let key = key.linux_keycode();

    let key_down = InputEvent {
        time,
        event_type: EV_KEY,
        code: key,
        value: 1,
    };

    let key_up = InputEvent {
        time,
        event_type: EV_KEY,
        code: key,
        value: 0,
    };

    let syn = InputEvent {
        time,
        event_type: EV_SYN,
        code: SYN_REPORT,
        value: 0,
    };

    keyboard.write_all(&key_down.bytes()).unwrap();
    keyboard.write_all(&syn.bytes()).unwrap();
    keyboard.write_all(&key_up.bytes()).unwrap();
    keyboard.write_all(&syn.bytes()).unwrap();
}

const SYN: InputEvent = InputEvent {
    time: Timeval {
        seconds: 0,
        microseconds: 0,
    },
    event_type: EV_SYN,
    code: SYN_REPORT,
    value: 0,
};

pub fn device_valid(mouse: &mut File) -> bool {
    if mouse.write_all(&SYN.bytes()).is_ok() {
        return true;
    }
    false
}

fn decompose_bits(bitmask: u64, index: usize) -> Vec<u64> {
    (0..64)
        .filter(|bit| (bitmask & (1 << bit)) != 0)
        .map(|bit| bit + index as u64 * 64) // Check if the bit is set
        .collect()
}
