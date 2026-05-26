use std::sync::Arc;

use utils::{channel::Channel, log::LoggerOptions, sync::Mutex};

use crate::{config::BASE_PATH, data::Data, os::mouse::check_uinput, ui::app::App};

mod config;
mod constants;
mod cs2;
mod data;
mod game;
mod math;
mod message;
mod os;
mod parser;
mod ui;

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

    // this runs as x11 for now, because wayland decorations for winit are not good
    // and don't support disabling the maximize button
    unsafe { std::env::remove_var("WAYLAND_DISPLAY") };

    let (channel_gui, channel_game) = Channel::new();
    let data = Arc::new(Mutex::new(Data::default()));
    let data_game = data.clone();

    std::thread::spawn(move || {
        game::GameManager::new(channel_game, data_game).run();
    });

    let event_loop = match winit::event_loop::EventLoop::new() {
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
