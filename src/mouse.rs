use std::{
    fs::{self, File, OpenOptions, read_dir},
    io::Write,
    os::unix::fs::FileTypeExt,
    time::{SystemTime, UNIX_EPOCH},
};

use glam::{IVec2, Vec2};

use crate::keyboard::VirtualKeyboard;

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
const REL_WHEEL: u16 = 0x08;
const BTN_LEFT: u16 = 0x110;

#[derive(Debug, Clone)]
pub struct MouseDevice {
    pub path: String,
    pub name: String,
    pub event_name: String,
}

impl MouseDevice {
    pub fn open(&self) -> Result<Mouse, std::io::Error> {
        let file = OpenOptions::new().write(true).open(&self.path)?;
        Ok(Mouse {
            file,
            keyboard: None,
            status: DeviceStatus::Working(self.name.clone()),
        })
    }

    pub fn try_open(&self) -> Mouse {
        match self.open() {
            Ok(mouse) => mouse,
            Err(_) => {
                log::warn!("please add your user to the input group or execute with sudo");
                log::warn!(
                    "without this, mouse movements will be written to /dev/null and discarded"
                );
                let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
                Mouse {
                    file,
                    keyboard: None,
                    status: DeviceStatus::PermissionsRequired,
                }
            }
        }
    }
}

pub struct Mouse {
    file: File,
    keyboard: Option<VirtualKeyboard>,
    pub status: DeviceStatus,
}


impl Mouse {
    pub fn open() -> Self {
        let devices = discover_mice();
        let keyboard = VirtualKeyboard::new();
        
        if let Some(device) = devices.first() {
            let mut mouse = device.try_open();
            mouse.keyboard = keyboard;
            mouse
        } else {
            let file = OpenOptions::new().write(true).open("/dev/null").unwrap();
            log::warn!("no mouse found");
            Mouse {
                file,
                keyboard,
                status: DeviceStatus::NotFound,
            }
        }
    }

    pub fn move_rel(&mut self, coords: &Vec2) {
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

        self.file.write_all(&x.bytes()).unwrap();
        self.file.write_all(&syn.bytes()).unwrap();

        self.file.write_all(&y.bytes()).unwrap();
        self.file.write_all(&syn.bytes()).unwrap();
    }

    pub fn left_press(&mut self) {
        self.key(1);
    }

    pub fn left_release(&mut self) {
        self.key(0);
    }

    pub fn scroll_wheel(&mut self, value: i32) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let wheel = InputEvent {
            time,
            event_type: EV_REL,
            code: REL_WHEEL,
            value,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        self.file.write_all(&wheel.bytes()).unwrap();
        self.file.write_all(&syn.bytes()).unwrap();
    }

    fn key(&mut self, pressed: i32) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let press = InputEvent {
            time,
            event_type: EV_KEY,
            code: BTN_LEFT,
            value: pressed,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        self.file.write_all(&press.bytes()).unwrap();
        self.file.write_all(&syn.bytes()).unwrap();
    }

    pub fn valid(&mut self) -> bool {
        self.file.write_all(&SYN.bytes()).is_ok()
    }

    pub fn press_a(&mut self) {
        if let Some(ref mut kbd) = self.keyboard {
            kbd.press_a();
        }
    }

    pub fn release_a(&mut self) {
        if let Some(ref mut kbd) = self.keyboard {
            kbd.release_a();
        }
    }

    pub fn press_d(&mut self) {
        if let Some(ref mut kbd) = self.keyboard {
            kbd.press_d();
        }
    }

    pub fn release_d(&mut self) {
        if let Some(ref mut kbd) = self.keyboard {
            kbd.release_d();
        }
    }
}

fn is_mouse_device(event_name: &str) -> bool {
    let capabilities_path = format!("/sys/class/input/{}/device/capabilities/rel", event_name);
    if let Ok(rel_caps) = fs::read_to_string(&capabilities_path)
        && let Ok(caps) = u64::from_str_radix(rel_caps.trim().trim_start_matches("0x"), 16)
    {
        const REL_X_BIT: u64 = 1 << 0;
        const REL_Y_BIT: u64 = 1 << 1;
        return (caps & REL_X_BIT) != 0 && (caps & REL_Y_BIT) != 0;
    }

    let device_name_path = format!("/sys/class/input/{}/device/name", event_name);
    if let Ok(device_name) = fs::read_to_string(&device_name_path) {
        let name_lower = device_name.to_lowercase();

        if name_lower.contains("mouse") {
            return true;
        }

        if name_lower.contains("keyboard")
            || name_lower.contains("power")
            || name_lower.contains("sleep")
            || name_lower.contains("lid")
            || name_lower.contains("video")
            || name_lower.contains("audio")
            || name_lower.contains("consumer control")
            || name_lower.contains("system control")
            || name_lower.contains("wireless")
            || name_lower.contains("rfkill")
        {
            return false;
        }
    }

    false
}

pub fn discover_mice() -> Vec<MouseDevice> {
    let mut devices = Vec::new();

    let Ok(entries) = read_dir("/dev/input") else {
        return devices;
    };

    for file in entries {
        let Ok(entry) = file else {
            continue;
        };
        if !entry.file_type().unwrap().is_char_device() {
            continue;
        }

        let name = entry.file_name().into_string().unwrap();
        if !name.starts_with("event") {
            continue;
        }

        if !is_mouse_device(&name) {
            continue;
        }

        let device_name = fs::read_to_string(format!("/sys/class/input/{}/device/name", name))
            .unwrap_or_else(|_| "Unknown Device".to_string())
            .trim()
            .to_string();

        let path = format!("/dev/input/{}", name);

        devices.push(MouseDevice {
            path,
            name: device_name,
            event_name: name,
        });
    }

    devices
}

pub fn get_mouse_by_name(name: &str) -> Option<MouseDevice> {
    discover_mice()
        .into_iter()
        .find(|device| device.name.to_lowercase().contains(&name.to_lowercase()))
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
