use glam::{Vec2, Vec3};
use shared::Data;

pub fn angles_from_vector(forward: &Vec3) -> Vec2 {
    let yaw = forward.y.atan2(forward.x).to_degrees();
    let pitch = (-forward.z)
        .atan2(Vec2::new(forward.x, forward.y).length())
        .to_degrees();

    Vec2::new(pitch, yaw)
}

pub fn angles_to_fov(view_angles: &Vec2, aim_angles: &Vec2) -> f32 {
    let delta = view_angles - aim_angles;

    let pitch = (delta.x + 180.0).rem_euclid(360.0) - 180.0;
    let yaw = (delta.y + 180.0).rem_euclid(360.0) - 180.0;

    Vec2::new(pitch.abs(), yaw.abs()).length()
}

pub fn vec2_clamp(vec: &mut Vec2) {
    if vec.x > 89.0 && vec.x <= 180.0 {
        vec.x = 89.0;
    }
    if vec.x > 180.0 {
        vec.x -= 360.0;
    }
    if vec.x < -89.0 {
        vec.x = -89.0;
    }
    vec.y = (vec.y + 180.0).rem_euclid(360.0) - 180.0;
}

pub const CYLINDER_SAMPLES: usize = 64;

pub fn world_to_screen(position: &Vec3, data: &Data) -> Option<egui::Pos2> {
    let vm = &data.view_matrix;
    let mut screen_position = Vec2::new(
        vm.x_axis.x * position.x
            + vm.x_axis.y * position.y
            + vm.x_axis.z * position.z
            + vm.x_axis.w,
        vm.y_axis.x * position.x
            + vm.y_axis.y * position.y
            + vm.y_axis.z * position.z
            + vm.y_axis.w,
    );

    let w = vm.w_axis.x * position.x
        + vm.w_axis.y * position.y
        + vm.w_axis.z * position.z
        + vm.w_axis.w;

    if w < 0.0001 {
        return None;
    }

    screen_position /= w;

    let half_size = Vec2::new(data.window_size.x * 0.5, data.window_size.y * 0.5);

    screen_position.x = half_size.x + 0.5 * screen_position.x * data.window_size.x + 0.5;
    screen_position.y = half_size.y - 0.5 * screen_position.y * data.window_size.y + 0.5;

    if screen_position.x < 0.0
        || screen_position.x > data.window_size.x
        || screen_position.y < 0.0
        || screen_position.y > data.window_size.y
    {
        return None;
    }

    Some(egui::pos2(screen_position.x, screen_position.y))
}

fn weighted_average_component(
    history: &std::collections::VecDeque<Vec2>,
    component: impl Fn(&Vec2) -> f32,
) -> f32 {
    if history.is_empty() {
        return 0.0;
    }

    let (sum, weight_sum) = history
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(s, w), (i, v)| {
            let weight = 1.0 + i as f32 * 0.15;
            (s + component(v) * weight, w + weight)
        });
    sum / weight_sum
}

pub fn compute_max_acceleration_component(
    history: &std::collections::VecDeque<Vec2>,
    component: impl Fn(&Vec2) -> f32,
    multiplier: f32,
    range: (f32, f32),
    fallback: f32,
) -> f32 {
    if history.len() < 3 {
        return fallback;
    }

    (weighted_average_component(history, component) * multiplier).clamp(range.0, range.1)
}

pub fn soft_clamp_acceleration(accel: f32, max_accel: f32, decay_rate: f32) -> f32 {
    if accel.abs() <= max_accel {
        return accel;
    }

    let excess = accel.abs() - max_accel;
    accel.signum() * (max_accel + excess * (-excess * decay_rate).exp())
}

pub fn record_acceleration(
    history: &mut std::collections::VecDeque<Vec2>,
    value: Vec2,
    max_size: usize,
) {
    if value.x.abs() < 25.0 && value.y.abs() < 25.0 {
        history.push_front(value.abs());
        if history.len() > max_size {
            history.pop_back();
        }
    }
}
