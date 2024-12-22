use std::{
    fs::{self, read_dir, File, OpenOptions},
    io::Write,
    os::unix::fs::FileTypeExt,
    time::{SystemTime, UNIX_EPOCH},
};

use glam::{IVec2, Vec2};

use crate::config::DEBUG_WITHOUT_MOUSE;

#[derive(Clone, Debug, PartialEq)]
pub enum MouseStatus {
    Working(String),
    Disconnected,
    PermissionsRequired,
    NoMouseFound,
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
// const BTN_LEFT: u16 = 0x110;
// const BTN_RIGHT: u16 = 0x111;

pub fn open_mouse() -> (File, MouseStatus) {
    if DEBUG_WITHOUT_MOUSE {
        let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
        return (file, MouseStatus::Working(String::from("/dev/null")));
    }
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
        // todo: key capability handling
        let rel: Vec<u32> =
            fs::read_to_string(format!("/sys/class/input/{}/device/capabilities/rel", name))
                .unwrap()
                .split_whitespace() // Handle multiple hex numbers
                .filter_map(|hex| u32::from_str_radix(hex, 16).ok()) // Parse each hex number
                .flat_map(decompose_bits) // Decompose into individual bits
                .collect();
        if !rel.contains(&(AXIS_X as u32)) || !rel.contains(&(AXIS_Y as u32)) {
            continue;
        }
        let device_name =
            fs::read_to_string(format!("/sys/class/input/{}/device/name", name)).unwrap();

        let path = format!("/dev/input/{}", name);
        let file = OpenOptions::new().write(true).open(path);
        match file {
            Ok(file) => return (file, MouseStatus::Working(device_name)),
            Err(_) => {
                println!("please add your user to the input group or execute with sudo.");
                println!(
                    "without this, mouse movements will be written to /dev/null and discarded."
                );
                let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
                return (file, MouseStatus::PermissionsRequired);
            }
        }
    }

    let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
    (file, MouseStatus::NoMouseFound)
}

pub fn mouse_move(mouse: &mut File, coords: Vec2) {
    let coords = IVec2::new(coords.x as i32, coords.y as i32);
    if DEBUG_WITHOUT_MOUSE {
        println!("moving mouse: ({} / {})", coords.x, coords.y);
        return;
    }

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

#[allow(unused)]
fn mouse_left_press(mouse: &mut File) {
    if DEBUG_WITHOUT_MOUSE {
        println!("pressing left mouse");
        return;
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let time = Timeval {
        seconds: now.as_secs(),
        microseconds: now.subsec_micros() as u64,
    };

    let key = InputEvent {
        time,
        event_type: EV_KEY,
        code: 0x110,
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

#[allow(unused)]
fn mouse_left_release(mouse: &mut File) {
    if DEBUG_WITHOUT_MOUSE {
        println!("releasing left mouse");
        return;
    }

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

const SYN: InputEvent = InputEvent {
    time: Timeval {
        seconds: 0,
        microseconds: 0,
    },
    event_type: EV_SYN,
    code: SYN_REPORT,
    value: 0,
};

pub fn mouse_valid(mouse: &mut File) -> bool {
    if mouse.write_all(&SYN.bytes()).is_ok() {
        return true;
    }
    false
}

fn decompose_bits(bitmask: u32) -> Vec<u32> {
    (0..32)
        .filter(|bit| (bitmask & (1 << bit)) != 0) // Check if the bit is set
        .collect()
}
