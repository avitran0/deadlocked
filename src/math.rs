use glam::{IVec4, Mat4, Vec2, Vec3};

pub fn angles_from_vector(forward: Vec3) -> Vec2 {
    let mut yaw;
    let mut pitch;

    // forward vector points up or down
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

    Vec2::new(pitch, yaw)
}

pub fn angles_to_fov(view_angles: Vec2, aim_angles: Vec2) -> f32 {
    let mut delta = view_angles - aim_angles;

    if delta.x > 180.0 {
        delta.x = 360.0 - delta.x;
    }
    delta.x = delta.x.abs();

    // clamp?
    delta.y = ((delta.y + 180.0) % 360.0 - 180.0).abs();

    delta.length()
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

pub fn world_to_screen(screen_size: IVec4, view_matrix: Mat4, position: Vec3) -> Option<Vec2> {
    let mut screen_position = Vec2::new(
        view_matrix.x_axis.x * position.x
            + view_matrix.x_axis.y * position.y
            + view_matrix.x_axis.z * position.z
            + view_matrix.x_axis.w,
        view_matrix.y_axis.x * position.x
            + view_matrix.y_axis.y * position.y
            + view_matrix.y_axis.z * position.z
            + view_matrix.y_axis.w,
    );

    let w = view_matrix.w_axis.x * position.x
        + view_matrix.w_axis.y * position.y
        + view_matrix.w_axis.z * position.z
        + view_matrix.w_axis.w;

    if w < 0.01 {
        return None;
    }

    screen_position.x /= w;
    screen_position.y /= w;

    let x = screen_size.z as f32 / 2.0;
    let y = screen_size.w as f32 / 2.0;

    screen_position.x = x + 0.5 * screen_position.x * screen_size.z as f32 + 0.5;
    screen_position.y = y - 0.5 * screen_position.y * screen_size.w as f32 + 0.5;

    Some(screen_position)
}
