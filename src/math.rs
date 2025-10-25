use glam::{Vec2, Vec3};

pub fn angles_from_vector(forward: &Vec3) -> Vec2 {
    let mut yaw;
    let mut pitch;

    if forward.x == 0.0 && forward.y == 0.0 {
        yaw = 0.0;
        pitch = if forward.z > 0.0 { 270.0 } else { 90.0 };
    } else {
        yaw = forward.y.atan2(forward.x).to_degrees();
        if yaw < 0.0 {
            yaw += 360.0;
        }

        pitch = (-forward.z)
            .atan2(Vec2::new(forward.x, forward.y).length())
            .to_degrees();
        if pitch < 0.0 {
            pitch += 360.0;
        }
    }

    Vec2::new(yaw, pitch)
}

pub fn angles_to_fov(view_angles: &Vec2, aim_angles: &Vec2) -> f32 {
    let view_yaw = view_angles.y.to_radians();
    let view_pitch = view_angles.x.to_radians();
    let aim_yaw = aim_angles.y.to_radians();
    let aim_pitch = aim_angles.x.to_radians();

    let view_forward = Vec3::new(
        view_pitch.cos() * view_yaw.cos(),
        view_pitch.cos() * view_yaw.sin(),
        -view_pitch.sin(),
    );

    let aim_forward = Vec3::new(
        aim_pitch.cos() * aim_yaw.cos(),
        aim_pitch.cos() * aim_yaw.sin(),
        -aim_pitch.sin(),
    );

    let dot = view_forward.dot(aim_forward).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
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
    vec.y = (vec.y + 180.0) % 360.0 - 180.0;
}

#[cfg(feature = "visuals")]
pub fn world_to_screen(position: &Vec3, data: &crate::data::Data) -> Option<egui::Pos2> {
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

    screen_position.x = half_size.x + screen_position.x * half_size.x;
    screen_position.y = half_size.y - screen_position.y * half_size.y;

    if screen_position.x < 0.0
        || screen_position.x > data.window_size.x
        || screen_position.y < 0.0
        || screen_position.y > data.window_size.y
    {
        return None;
    }

    Some(egui::pos2(screen_position.x, screen_position.y))
}
