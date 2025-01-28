use glam::{vec2, Vec2, Vec3};
use rand::{rng, Rng};

// t will be config.smooth
pub fn jitter(aim_coords: Vec2, smooth: f32) -> Vec2 {
    let mut rng = rng();
    let smooth = aim_coords / smooth;
    let shared_noise = rng.random_range(-0.2..=0.2);
    let jitter = vec2(
        rng.random_range(-0.5..=0.5) * smooth.x,
        rng.random_range(-0.5..=0.5) * smooth.y,
    ) + shared_noise;
    smooth + jitter
}

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
