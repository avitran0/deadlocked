use std::{
    fs::File,
    io::Write,
    os::fd::AsRawFd,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use glam::Vec2;
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr, libc::c_ulong};
use utils::log;

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

#[repr(C)]
struct DeviceSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

const DEVICE_SETUP: DeviceSetup = DeviceSetup {
    id: InputId {
        // usb
        bustype: 0x03,
        // texas instruments
        vendor: 0x0451,
        // ti-84 silver
        // yes, this is a calculator, sending mouse inputs
        product: 0xe008,
        version: 1,
    },
    // "TI-84 Plus Silver Calculator"
    name: [
        84, 73, 45, 56, 52, 32, 80, 108, 117, 115, 32, 83, 105, 108, 118, 101, 114, 32, 67, 97,
        108, 99, 117, 108, 97, 116, 111, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0,
    ],
    ff_effects_max: 0,
};

const UINPUT_IOCTL_BASE: c_ulong = b'U' as c_ulong;
ioctl_none!(ui_dev_create, UINPUT_IOCTL_BASE, 1);
ioctl_none!(ui_dev_destroy, UINPUT_IOCTL_BASE, 2);
ioctl_write_int!(ui_set_evbit, UINPUT_IOCTL_BASE, 100);
ioctl_write_int!(ui_set_keybit, UINPUT_IOCTL_BASE, 101);
ioctl_write_int!(ui_set_relbit, UINPUT_IOCTL_BASE, 102);
ioctl_write_ptr!(ui_dev_setup, UINPUT_IOCTL_BASE, 3, DeviceSetup);

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const SYN_REPORT: u16 = 0x00;
const AXIS_X: u16 = 0x00;
const AXIS_Y: u16 = 0x01;
const BTN_LEFT: u16 = 0x110;

pub struct Mouse {
    file: File,
    remainder: Vec2,
}

static CREATED: AtomicBool = AtomicBool::new(false);
impl Mouse {
    pub fn open() -> anyhow::Result<Self> {
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err(anyhow::anyhow!("mouse already initialized"));
        }
        let file = File::options().write(true).open("/dev/uinput")?;
        let fd = file.as_raw_fd();

        unsafe {
            // enable event types
            ui_set_evbit(fd, EV_SYN as u64)?;
            ui_set_evbit(fd, EV_KEY as u64)?;
            ui_set_evbit(fd, EV_REL as u64)?;

            ui_set_relbit(fd, AXIS_X as u64)?;
            ui_set_relbit(fd, AXIS_Y as u64)?;

            ui_set_keybit(fd, BTN_LEFT as u64)?;

            ui_dev_setup(fd, &DEVICE_SETUP)?;
            ui_dev_create(fd)?;
        }

        Ok(Self {
            file,
            remainder: Vec2::ZERO,
        })
    }

    pub fn move_rel(&mut self, coords: &Vec2) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let time = Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        };

        let x = InputEvent {
            time,
            event_type: EV_REL,
            code: AXIS_X,
            value: coords.x as i32,
        };

        let y = InputEvent {
            time,
            event_type: EV_REL,
            code: AXIS_Y,
            value: coords.y as i32,
        };

        let syn = InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        };

        self.file.write_all(&x.bytes()).unwrap();
        self.file.write_all(&y.bytes()).unwrap();
        self.file.write_all(&syn.bytes()).unwrap();
    }

    pub fn move_rel_humanized(&mut self, coords: &Vec2, smooth: f32) {
        use rand::RngExt;

        let mut rng = rand::rng();

        // sub-pixel precision
        let total = *coords + self.remainder;
        let mut x = total.x;
        let mut y = total.y;

        // add some jitter if smooth is high enough
        if smooth > 1.0 {
            let jitter_factor = (smooth / 20.0).clamp(0.0, 1.0) * 0.5;
            x += rng.random_range(-jitter_factor..jitter_factor);
            y += rng.random_range(-jitter_factor..jitter_factor);
        }

        let ix = x as i32;
        let iy = y as i32;

        self.remainder = Vec2::new(x - ix as f32, y - iy as f32);

        if ix == 0 && iy == 0 {
            return;
        }

        self.move_rel(&Vec2::new(ix as f32, iy as f32));
    }

    pub fn left_press(&mut self) {
        self.key(1);
    }

    pub fn left_release(&mut self) {
        self.key(0);
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
}

impl Drop for Mouse {
    fn drop(&mut self) {
        let _ = unsafe { ui_dev_destroy(self.file.as_raw_fd()) };
        CREATED.store(false, Ordering::Relaxed);
    }
}

pub fn check_uinput() -> bool {
    let path = Path::new("/dev/uinput");
    if !path.exists() {
        log::error!("the uinput kernel module is not loaded.");
        log::error!("this module needs to be loaded for mouse input to work.");
        log::error!("please carefully read the readme before using.");
        return false;
    }
    if File::options().write(true).open(path).is_err() {
        log::error!("user has no write permissions for /dev/uinput.");
        log::error!("did you add your user to the input group?");
        log::error!("please carefully read the readme before using.");
        return false;
    }
    true
}
