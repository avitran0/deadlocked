use std::{
    io::Write,
    path::Path,
    sync::{mpsc, Arc},
    thread,
};

use color::Colors;
use config::{get_config_path, parse_config};
use eframe::egui::{self, FontData, FontDefinitions, Stroke, Style};
use message::{Game, Message};
use notify::{
    event::{DataChange, ModifyKind},
    EventKind, Watcher,
};

mod aimbot;
mod color;
mod config;
mod constants;
mod cs2;
mod gui;
mod key_codes;
mod math;
mod message;
mod mouse;
mod proc;
mod process;

#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported.");

fn main() {
    let env = env_logger::Env::new();
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter_level(log::LevelFilter::Off)
        .filter_module("deadlocked", log::LevelFilter::Info)
        .parse_env(env)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let headless = args.len() > 1 && args[1] == "--headless";

    // this runs as x11 for now, because wayland decorations for winit are not good
    // and don't support disabling the maximize button
    std::env::remove_var("WAYLAND_DISPLAY");

    let username = std::env::var("USER").unwrap_or_default();
    if username == "root" {
        println!("start without sudo, and add your user to the input group.");
        return;
    }

    let (tx_aimbot, rx_gui) = mpsc::channel();
    let (tx_gui, rx_aimbot) = mpsc::channel();

    thread::Builder::new()
        .name(String::from("deadlocked"))
        .spawn(move || {
            aimbot::AimbotManager::new(tx_aimbot, rx_aimbot).run();
        })
        .expect("could not create aimbot thread");

    if headless {
        let mut watcher =
            notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                if let Ok(event) = event {
                    if event.kind != EventKind::Modify(ModifyKind::Data(DataChange::Any)) {
                        return;
                    }
                    let config = parse_config();
                    tx_gui
                        .send(Message::Config(match config.current_game {
                            Game::CS2 => config.cs2,
                            Game::Deadlock => config.deadlock,
                        }))
                        .unwrap();
                }
            })
            .unwrap();
        watcher
            .watch(
                Path::new(&get_config_path()),
                notify::RecursiveMode::NonRecursive,
            )
            .unwrap();

        loop {
            let _ = rx_gui.recv().expect("aimbot thread died");
        }
    }
    let window_size = [600.0, 350.0];
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximize_button(false)
            .with_inner_size(window_size)
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "deadlocked",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_pixels_per_point(1.5);

            let font = include_bytes!("../resources/Nunito.ttf");
            let mut font_definitions = FontDefinitions::default();
            font_definitions.font_data.insert(
                String::from("nunito"),
                Arc::new(FontData::from_static(font)),
            );

            font_definitions
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, String::from("nunito"));

            cc.egui_ctx.set_fonts(font_definitions);

            cc.egui_ctx.style_mut_of(egui::Theme::Dark, gui_style);

            Ok(Box::new(gui::Gui::new(tx_gui, rx_gui)))
        }),
    )
    .unwrap();
}

fn gui_style(style: &mut Style) {
    style.interaction.selectable_labels = false;
    //style.visuals.override_text_color = Some(Color32::WHITE);

    style.visuals.window_fill = Colors::BASE;
    style.visuals.panel_fill = Colors::BASE;
    style.visuals.extreme_bg_color = Colors::BACKDROP;

    let bg_stroke = Stroke::new(1.0, Colors::SUBTEXT);
    let fg_stroke = Stroke::new(1.0, Colors::TEXT);
    let dark_stroke = Stroke::new(1.0, Colors::BASE);

    style.visuals.selection.bg_fill = Colors::BLUE;
    style.visuals.selection.stroke = dark_stroke;

    style.visuals.widgets.active.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.active.bg_stroke = bg_stroke;
    style.visuals.widgets.active.fg_stroke = fg_stroke;
    style.visuals.widgets.active.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.hovered.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.hovered.bg_stroke = bg_stroke;
    style.visuals.widgets.hovered.fg_stroke = fg_stroke;
    style.visuals.widgets.hovered.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.inactive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.inactive.fg_stroke = fg_stroke;
    style.visuals.widgets.inactive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.noninteractive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.noninteractive.fg_stroke = fg_stroke;
    style.visuals.widgets.noninteractive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.open.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.open.bg_stroke = bg_stroke;
    style.visuals.widgets.open.fg_stroke = fg_stroke;
    style.visuals.widgets.open.weak_bg_fill = Colors::HIGHLIGHT;
}
