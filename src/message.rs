use crate::{
    config::{AimbotStatus, Config},
    input_device::DeviceStatus,
};

#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    Status(AimbotStatus),
    MouseStatus(DeviceStatus),
    // KeyboardStatus(DeviceStatus),
    FrameTime(f64),
}
