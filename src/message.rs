use crate::{
    config::{AimbotStatus, Config},
    mouse::MouseStatus,
};

#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    Status(AimbotStatus),
    MouseStatus(MouseStatus),
    FrameTime(f64),
}
