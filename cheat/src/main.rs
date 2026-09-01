use std::sync::Arc;

use shared::Data;
use utils::{Channel, Mutex, log::LoggerOptions};
use winit::platform::x11::EventLoopBuilderExtX11;

use crate::{config::BASE_PATH, os::mouse::check_uinput, ui::app::App};

mod config;
mod constants;
mod cs2;
mod font;
mod game;
mod math;
mod message;
mod os;
mod parser;
mod radar;
mod ui;
mod update;

#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported.");

fn main() {
    utils::log::init(
        LoggerOptions::default()
            .file(BASE_PATH.join("deadlocked.log"))
            .truncate(true),
        |w, rec| {
            writeln!(
                w,
                "[{}] [{}:{}] {}",
                rec.level, rec.location.file, rec.location.line, rec.args
            )
        },
    )
    .expect("failed to initialize logger");

    if !check_uinput() {
        return;
    }

    let (channel_gui, channel_game) = Channel::new();
    let (channel_gui_radar, channel_radar) = Channel::new();
    let data = Arc::new(Mutex::new(Data::default()));
    let data_game = data.clone();
    let data_radar = data.clone();

    std::thread::spawn(move || {
        game::GameManager::new(channel_game, data_game).run();
    });

    std::thread::spawn(move || {
        radar::Radar::new(channel_radar, data_radar).run();
    });

    let event_loop = match winit::event_loop::EventLoop::builder().with_x11().build() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            utils::error!("failed to create event loop: {err}");
            return;
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(channel_gui, data);
    event_loop.run_app(&mut app).unwrap();
}
