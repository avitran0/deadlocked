use std::sync::Arc;

use crossbeam::channel::{bounded, unbounded};
use utils::{
    log::{self, Logger, LoggerOptions},
    sync::Mutex,
};

use crate::{
    data::Data,
    os::mouse::check_uinput,
    parser::parse_maps,
    ui::{app::App, grenades::read_grenades},
};

mod config;
mod constants;
mod cs2;
mod data;
mod game;
mod math;
mod message;
mod os;
mod parser;
mod radar;
mod router;
mod ui;

#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported.");

fn main() {
    Logger::install(
        LoggerOptions::default()
            .file("deadlocked.log")
            .debug(true)
            .truncate(true)
            .module(module_path!()),
    );

    let args: Vec<String> = std::env::args().collect();
    os::crash::install_crash_handler();

    if !check_uinput() {
        return;
    }

    let force_reparse = args.iter().any(|arg| arg == "--force-reparse");
    let use_system_binary = args.iter().any(|arg| arg == "--local-s2v");
    let force_x11 = args.iter().any(|arg| arg == "--x11");

    if force_x11 {
        unsafe { std::env::remove_var("WAYLAND_DISPLAY") };
    }
    spawn_with_crash_handler(move || {
        parse_maps(force_reparse, use_system_binary);
    });

    let (tx, rx) = unbounded();
    let (tx_gui, rx_gui) = bounded(16);
    let (tx_game, rx_game) = bounded(16);
    let (tx_radar, rx_radar) = bounded(16);
    let data = Arc::new(Mutex::new(Data::default()));
    let data_game = data.clone();
    let data_radar = data.clone();
    let grenades = Arc::new(Mutex::new(read_grenades()));
    let grenades_game = grenades.clone();

    spawn_with_crash_handler(move || {
        router::router(rx, tx_gui, tx_game, tx_radar);
    });

    let tx_game = tx.clone();
    spawn_with_crash_handler(move || {
        game::GameManager::new(tx_game, rx_game, data_game, grenades_game).run();
    });

    let tx_radar = tx.clone();
    spawn_with_crash_handler(move || {
        radar::Radar::new(tx_radar, rx_radar, data_radar).run();
    });

    let event_loop = match winit::event_loop::EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            log::error!("failed to create event loop: {err}");
            return;
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(tx, rx_gui, data, grenades);
    event_loop.run_app(&mut app).unwrap();
}

fn spawn_with_crash_handler<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::spawn(move || {
        os::crash::install_crash_handler();
        f();
    });
}
