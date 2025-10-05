use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::fd::AsRawFd,
    time::{SystemTime, UNIX_EPOCH},
};

use nix::ioctl_write_int_bad;

// Linux input event keycodes
const KEY_A: u16 = 30;
const KEY_D: u16 = 32;

// Event types
const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const SYN_REPORT: u16 = 0x00;

// uinput constants
const UINPUT_MAX_NAME_SIZE: usize = 80;
const UI_SET_EVBIT: u64 = 0x40045564;
const UI_SET_KEYBIT: u64 = 0x40045565;
const UI_DEV_SETUP: u64 = 0x405c5503;
const UI_DEV_CREATE: u64 = 0x5501;
const UI_DEV_DESTROY: u64 = 0x5502;
const BUS_USB: u16 = 0x03;

ioctl_write_int_bad!(ui_set_evbit, UI_SET_EVBIT);
ioctl_write_int_bad!(ui_set_keybit, UI_SET_KEYBIT);
ioctl_write_int_bad!(ui_dev_create, UI_DEV_CREATE);
ioctl_write_int_bad!(ui_dev_destroy, UI_DEV_DESTROY);

use nix::ioctl_write_ptr_bad;
ioctl_write_ptr_bad!(ui_dev_setup, UI_DEV_SETUP, UinputSetup);

#[repr(C)]
struct UinputSetup {
    id: InputId,
    name: [u8; UINPUT_MAX_NAME_SIZE],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
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

pub struct VirtualKeyboard {
    device: File,
}

impl VirtualKeyboard {
    pub fn new() -> Option<Self> {
        let device = create_virtual_keyboard()?;
        Some(Self { device })
    }

    fn send_key(&mut self, code: u16, pressed: i32) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let key_event = InputEvent {
            time,
            event_type: EV_KEY,
            code,
            value: pressed,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        if let Err(e) = self.device.write_all(&key_event.bytes()) {
            log::warn!("failed to write key event: {}", e);
            return;
        }
        if let Err(e) = self.device.write_all(&syn.bytes()) {
            log::warn!("failed to write syn event: {}", e);
        }
    }

    pub fn press_a(&mut self) {
        self.send_key(KEY_A, 1);
    }

    pub fn release_a(&mut self) {
        self.send_key(KEY_A, 0);
    }

    pub fn press_d(&mut self) {
        self.send_key(KEY_D, 1);
    }

    pub fn release_d(&mut self) {
        self.send_key(KEY_D, 0);
    }
}

impl Drop for VirtualKeyboard {
    fn drop(&mut self) {
        unsafe {
            let _ = ui_dev_destroy(self.device.as_raw_fd(), 0);
        }
    }
}

fn create_virtual_keyboard() -> Option<File> {
    let uinput_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/uinput")
        .or_else(|_| OpenOptions::new().read(true).write(true).open("/dev/input/uinput"));

    let file = match uinput_file {
        Ok(f) => f,
        Err(e) => {
            log::warn!("failed to open uinput: {} (strafe helper won't work)", e);
            return None;
        }
    };

    let fd = file.as_raw_fd();

    unsafe {
        if ui_set_evbit(fd, EV_SYN as i32).is_err() {
            log::warn!("failed to set EV_SYN");
            return None;
        }

        if ui_set_evbit(fd, EV_KEY as i32).is_err() {
            log::warn!("failed to set EV_KEY");
            return None;
        }

        if ui_set_keybit(fd, KEY_A as i32).is_err() || ui_set_keybit(fd, KEY_D as i32).is_err() {
            log::warn!("failed to set key bits");
            return None;
        }

        let mut setup = UinputSetup {
            id: InputId {
                bustype: BUS_USB,
                vendor: 0x1234,
                product: 0x5678,
                version: 1,
            },
            name: [0u8; UINPUT_MAX_NAME_SIZE],
            ff_effects_max: 0,
        };

        let name_bytes = b"deadlocked-virtual-kbd";
        let copy_len = name_bytes.len().min(UINPUT_MAX_NAME_SIZE - 1);
        setup.name[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

        if ui_dev_setup(fd, &setup).is_err() {
            log::warn!("failed to setup uinput device");
            return None;
        }

        if ui_dev_create(fd, 0).is_err() {
            log::warn!("failed to create uinput device");
            return None;
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    log::info!("created virtual keyboard device");
    Some(file)
}
