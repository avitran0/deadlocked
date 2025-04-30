use std::time::Duration;

use crate::{
    config::{AimbotStatus, Config},
    mouse::DeviceStatus,
};

#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    Status(AimbotStatus),
    MouseStatus(DeviceStatus),
    FrameTime(Duration),
}
