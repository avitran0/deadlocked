use std::{
    collections::HashMap,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use femtovg::{renderer::OpenGl, Canvas, Color, ImageId, LineCap, Paint, Path};
use glam::{vec2, vec4, IVec4, Mat4, Vec2};
use log::{info, warn};
use sdl3::{event::Event, rect::Point, render::FPoint, video::Window};

use crate::{
    config::VisualsConfig,
    icons,
    math::world_to_screen,
    message::{DrawStyle, EntityInfo, PlayerInfo, VisualsMessage},
};

#[derive(Debug, Default)]
struct DrawInfo {
    view_matrix: Mat4,
    window_size: IVec4,
}

pub fn visuals(rx: Receiver<VisualsMessage>) {
    let context = sdl3::init().unwrap();
    let video = context.video().unwrap();

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let mut display_min = Point::new(i32::MAX, i32::MAX);
    let mut display_max = Point::new(0, 0);
    for i in 0..video.num_video_drivers().unwrap() {
        if let Ok(bounds) = video.display_bounds(i as u32) {
            let x = bounds.x + bounds.w;
            if bounds.x < display_min.x {
                display_min.x = bounds.x;
            }
            if x > display_max.x {
                display_max.x = x;
            }

            let y = bounds.y + bounds.h;
            if bounds.y < display_min.y {
                display_min.y = bounds.y;
            }
            if y > display_max.y {
                display_max.y = y;
            }
        }
    }
    info!(
        "screen top left corner at: {} x {} px",
        display_min.x, display_min.y
    );
    info!(
        "screen resolution detected: {} x {} px",
        display_max.x, display_max.y
    );

    // position + size
    let display = vec4(
        display_min.x as f32,
        display_min.y as f32,
        (display_max.x - display_min.x) as f32,
        (display_max.y - display_min.y) as f32,
    );

    let w = video
        .window("deadlocked", 1, 1)
        .borderless()
        //.hidden()
        .minimized()
        .position(0, 0)
        .build()
        .unwrap();
    let mut window = unsafe {
        video
            .popup_window(
                &w,
                (display_max.x - display_min.x) as u32,
                (display_max.y - display_min.y) as u32,
            )
            .always_on_top()
            .tooltip()
            .transparent()
            .opengl()
            .build()
            .unwrap()
    };
    window.set_position(
        sdl3::video::WindowPos::Positioned(display_min.x),
        sdl3::video::WindowPos::Positioned(display_min.y),
    );
    window.set_opacity(0.0).expect("could not set opacity");

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    video.gl_set_swap_interval(0).unwrap();

    let renderer = unsafe {
        OpenGl::new_from_function(|s| video.gl_get_proc_address(s).unwrap() as *const _).unwrap()
    };
    let mut canvas = Canvas::new(renderer).unwrap();
    let size = glam::uvec2(
        (display_max.x - display_min.x) as u32,
        (display_max.y - display_min.y) as u32,
    );
    canvas.set_size(size.x, size.y, 1.0);

    let icons = icons::init(&mut canvas);

    let mut event_pump = context.event_pump().unwrap();
    let mut player_info = vec![];
    let mut entity_info = vec![];
    let mut draw_info = DrawInfo::default();
    let mut config = VisualsConfig::default();
    let transparent = Color::rgba(0, 0, 0, 0);
    'running: loop {
        let start = Instant::now();
        while let Ok(message) = rx.try_recv() {
            match message {
                VisualsMessage::PlayerInfo(info) => player_info = info,
                VisualsMessage::EntityInfo(info) => entity_info = info,
                VisualsMessage::ViewMatrix(matrix) => draw_info.view_matrix = matrix,
                VisualsMessage::WindowSize(size) => draw_info.window_size = size,

                VisualsMessage::EnableVisuals(enabled) => config.enabled = enabled,
                VisualsMessage::DrawBox(draw_box) => config.draw_box = draw_box,
                VisualsMessage::BoxColor(color) => config.box_color = color,
                VisualsMessage::DrawSkeleton(draw_skeleton) => config.draw_skeleton = draw_skeleton,
                VisualsMessage::SkeletonColor(color) => config.skeleton_color = color,
                VisualsMessage::DrawHealth(draw_health) => config.draw_health = draw_health,
                VisualsMessage::DrawArmor(draw_armor) => config.draw_armor = draw_armor,
                VisualsMessage::ArmorColor(color) => config.armor_color = color,
                VisualsMessage::DrawWeapon(draw_weapon) => config.draw_weapon = draw_weapon,
                VisualsMessage::DroppedItems(dropped_items) => config.dropped_items = dropped_items,
                VisualsMessage::VisibilityCheck(visibility_check) => {
                    config.visibility_check = visibility_check
                }
                VisualsMessage::VisualsFps(fps) => config.fps = fps,
                VisualsMessage::DebugWindow(debug) => config.debug_window = debug,
                VisualsMessage::Config(c) => config = c,
                VisualsMessage::Quit => break 'running,
            }
        }

        canvas.clear_rect(0, 0, size.x, size.y, transparent);
        for event in event_pump.poll_iter() {
            if let Event::Quit { timestamp: _ } = event {
                break 'running;
            }
        }

        if !config.enabled {
            end(&mut canvas, &window, start, Duration::from_millis(30));
            continue;
        }

        if config.debug_window {
            let width = 4.0;
            let mut debug_lines = Path::new();
            debug_lines.rect(
                display.x + width / 2.0,
                display.y + width / 2.0,
                display.z - width,
                display.w - width,
            );
            debug_lines.move_to(display.x, display.y);
            debug_lines.line_to(display.x + display.z, display.y + display.w);
            debug_lines.move_to(display.x, display.y + display.w);
            debug_lines.line_to(display.x + display.z, display.y);
            canvas.stroke_path(
                &debug_lines,
                &Paint::color(Color::white()).with_line_width(width),
            );
        }

        for player in &player_info {
            if !player.visible && config.visibility_check {
                continue;
            }

            let line_width = draw_box(&mut canvas, &config, &draw_info, &icons, player);
            draw_skeleton(&mut canvas, &config, &draw_info, player, line_width);
        }

        for entity in &entity_info {
            draw_entity(&mut canvas, &config, &draw_info, &icons, entity);
        }

        end(
            &mut canvas,
            &window,
            start,
            Duration::from_micros(1_000_000 / config.fps),
        );
    }
}

fn draw_box(
    canvas: &mut Canvas<OpenGl>,
    config: &VisualsConfig,
    draw_info: &DrawInfo,
    icons: &HashMap<&str, ImageId>,
    player: &PlayerInfo,
) -> f32 {
    let box_color = match config.draw_box {
        DrawStyle::None => Color::white(),
        DrawStyle::Color => config.box_color.femtovg_color(),
        DrawStyle::Health => get_health_color(player.health),
    };

    let screen_position = vec2(
        draw_info.window_size.x as f32,
        draw_info.window_size.y as f32,
    );

    let position = match world_to_screen(
        draw_info.window_size,
        draw_info.view_matrix,
        player.position,
    ) {
        Some(pos) => pos + screen_position,
        None => return 1.0,
    };

    let mut head_vec = player.position;
    head_vec.z += (player.head.z - player.position.z).abs() + 8.0;
    let head_position =
        match world_to_screen(draw_info.window_size, draw_info.view_matrix, head_vec) {
            Some(pos) => vec2(position.x, pos.y + screen_position.y),
            None => return 1.0,
        };

    let height = (head_position.y - position.y).abs();
    let width = height / 2.0;

    let line_width = width / 4.0;
    let line_height = height / 4.0;

    let top_left = vec2(head_position.x - width / 2.0, head_position.y);
    let top_right = vec2(head_position.x + width / 2.0, head_position.y);
    let bottom_left = vec2(position.x - width / 2.0, position.y);
    let bottom_right = vec2(position.x + width / 2.0, position.y);

    let draw_box = config.draw_box != DrawStyle::None;
    let mut path = Path::new();

    if draw_box && is_on_screen(top_left, draw_info) {
        path.move_to(top_left.x, top_left.y);
        path.line_to(top_left.x + line_width, top_left.y);

        path.move_to(top_left.x, top_left.y);
        path.line_to(top_left.x, top_left.y + line_height);
    }

    if draw_box && is_on_screen(top_right, draw_info) {
        path.move_to(top_right.x, top_right.y);
        path.line_to(top_right.x - line_width, top_right.y);

        path.move_to(top_right.x, top_right.y);
        path.line_to(top_right.x, top_right.y + line_height);
    }

    if draw_box && is_on_screen(bottom_left, draw_info) {
        path.move_to(bottom_left.x, bottom_left.y);
        path.line_to(bottom_left.x + line_width, bottom_left.y);

        path.move_to(bottom_left.x, bottom_left.y);
        path.line_to(bottom_left.x, bottom_left.y - line_height);
    }

    if draw_box && is_on_screen(bottom_right, draw_info) {
        path.move_to(bottom_right.x, bottom_right.y);
        path.line_to(bottom_right.x - line_width, bottom_right.y);

        path.move_to(bottom_right.x, bottom_right.y);
        path.line_to(bottom_right.x, bottom_right.y - line_height);
    }

    canvas.stroke_path(
        &path,
        &Paint::color(box_color)
            .with_anti_alias(true)
            .with_line_cap(LineCap::Round)
            .with_line_width((line_width / 4.0).clamp(1.0, 2.0)),
    );

    // health bar
    let bar_width = (line_width / 8.0).clamp(1.0, 8.0);
    if config.draw_health
        && is_on_screen(bottom_left, draw_info)
        && is_on_screen(top_left, draw_info)
    {
        let mut path = Path::new();
        let bottom_left = FPoint::new(bottom_left.x - bar_width as f32 * 2.0, bottom_left.y);
        let bar_height = height * (player.health as f32 / 100.0);

        path.rect(bottom_left.x, bottom_left.y, bar_width, -bar_height);
        canvas.fill_path(
            &path,
            &Paint::color(get_health_color(player.health))
                .with_anti_alias(true)
                .with_line_cap(LineCap::Round),
        );
    }

    // armor bar
    if config.draw_armor
        && player.armor > 0
        && is_on_screen(bottom_left, draw_info)
        && is_on_screen(top_left, draw_info)
    {
        let mut path = Path::new();
        let bottom_left = FPoint::new(bottom_left.x - bar_width as f32 * 4.0, bottom_left.y);
        let bar_height = height * (player.armor as f32 / 100.0);

        path.rect(bottom_left.x, bottom_left.y, bar_width, -bar_height);
        canvas.fill_path(
            &path,
            &Paint::color(config.armor_color.femtovg_color())
                .with_anti_alias(true)
                .with_line_cap(LineCap::Round),
        );
    }

    // weapon icon
    if config.draw_weapon && is_on_screen(bottom_left, draw_info) {
        if let Some(icon) = icons.get(player.weapon.as_str()) {
            let (icon_width, icon_height) = canvas.image_size(*icon).unwrap();
            let scale_factor = (height / 1200.0).clamp(0.025, 0.25);
            let mut path = Path::new();
            path.rect(
                bottom_left.x,
                bottom_left.y + 4.0,
                icon_width as f32 * scale_factor,
                icon_height as f32 * scale_factor,
            );
            canvas.fill_path(
                &path,
                &Paint::image(
                    *icon,
                    bottom_left.x,
                    bottom_left.y + 4.0,
                    icon_width as f32 * scale_factor,
                    icon_height as f32 * scale_factor,
                    0.0,
                    1.0,
                ),
            );
        }
    }

    (line_width / 4.0).clamp(1.0, 2.0)
}

fn draw_skeleton(
    canvas: &mut Canvas<OpenGl>,
    config: &VisualsConfig,
    draw_info: &DrawInfo,
    player: &PlayerInfo,
    line_width: f32,
) {
    let skeleton_color = match config.draw_skeleton {
        DrawStyle::None => return,
        DrawStyle::Color => config.skeleton_color.femtovg_color(),
        DrawStyle::Health => get_health_color(player.health),
    };
    let screen_position = vec2(
        draw_info.window_size.x as f32,
        draw_info.window_size.y as f32,
    );
    let mut path = Path::new();
    for connection in &player.bones {
        let bone1 =
            match world_to_screen(draw_info.window_size, draw_info.view_matrix, connection.0) {
                Some(pos) => pos + screen_position,
                None => continue,
            };
        let bone2 =
            match world_to_screen(draw_info.window_size, draw_info.view_matrix, connection.1) {
                Some(pos) => pos + screen_position,
                None => continue,
            };

        if bone1.x < 1.0 && bone1.y < 1.0 {
            continue;
        }
        if bone2.x < 1.0 && bone2.y < 1.0 {
            continue;
        }

        if is_on_screen(bone1, draw_info) && is_on_screen(bone2, draw_info) {
            path.move_to(bone1.x, bone1.y);
            path.line_to(bone2.x, bone2.y);
        }
    }
    canvas.stroke_path(
        &path,
        &Paint::color(skeleton_color)
            .with_anti_alias(true)
            .with_line_cap(LineCap::Round)
            .with_line_width(line_width),
    );
}

fn draw_entity(
    canvas: &mut Canvas<OpenGl>,
    config: &VisualsConfig,
    draw_info: &DrawInfo,
    icons: &HashMap<&str, ImageId>,
    entity: &EntityInfo,
) {
    if !config.dropped_items {
        return;
    }
    let screen_position = vec2(
        draw_info.window_size.x as f32,
        draw_info.window_size.y as f32,
    );
    let position = match world_to_screen(
        draw_info.window_size,
        draw_info.view_matrix,
        entity.position,
    ) {
        Some(pos) => pos + screen_position,
        None => return,
    };
    if !is_on_screen(position, draw_info) {
        return;
    }
    if let Some(icon) = icons.get(entity.name.as_str()) {
        let (icon_width, icon_height) = canvas.image_size(*icon).unwrap();
        let mut path = Path::new();
        let scale = (50.0 / entity.distance).clamp(0.1, 0.2);
        let size = vec2(icon_width as f32, icon_height as f32) * scale;
        path.rect(
            position.x - size.x / 2.0,
            position.y - size.y / 2.0,
            size.x,
            size.y,
        );
        canvas.fill_path(
            &path,
            &Paint::image(
                *icon,
                position.x - size.x / 2.0,
                position.y - size.y / 2.0,
                size.x,
                size.y,
                0.0,
                1.0,
            ),
        );
    }
}

fn end(canvas: &mut Canvas<OpenGl>, window: &Window, start: Instant, dur: Duration) {
    canvas.flush();
    window.gl_swap_window();
    let elapsed = start.elapsed();
    if elapsed < dur {
        std::thread::sleep(dur - elapsed);
    } else {
        warn!("visuals loop took {}ms", elapsed.as_millis());
        std::thread::sleep(dur);
    }
}

fn is_on_screen(point: Vec2, draw_info: &DrawInfo) -> bool {
    let window_size = draw_info.window_size.as_vec4();
    if point.x < window_size.x
        || point.y < window_size.y
        || point.x > window_size.x + window_size.z
        || point.y > window_size.y + window_size.w
    {
        return false;
    }
    true
}

fn get_health_color(health: i32) -> Color {
    let health = health.clamp(0, 100);

    let (r, g, b) = if health <= 50 {
        let factor = health as f32 / 50.0;
        (255, (255.0 * factor) as u8, 0)
    } else {
        let factor = (health as f32 - 50.0) / 50.0;
        ((255.0 * (1.0 - factor)) as u8, 255, 0)
    };

    Color::rgb(r, g, b)
}
