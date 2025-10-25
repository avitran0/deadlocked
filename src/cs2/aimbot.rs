use glam::Vec2;
use crate::{
    cs2::{CS2, entity::player::Player},
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

// ---------- helpers ---------------------------------------------------------
fn human_curve(x: f32) -> f32 { x * x * (3.0 - 2.0 * x) }

fn white(seed: u32) -> f32 {
    let s = (seed << 13) ^ seed;
    let s = s.wrapping_mul(s.wrapping_mul(15731).wrapping_add(789221));
    ((s >> 16) & 0x7fff) as f32 / 16384.0 - 1.0
}

// ---------- aimbot ----------------------------------------------------------
impl CS2 {
    pub fn aimbot(&mut self, _config: &crate::config::Config, mouse: &mut Mouse) {
        // pull the same config you already expose
        let cfg = self.aimbot_config(_config);
        if !cfg.enabled || self.target.player.is_none() { return; }
        let target = self.target.player.as_ref().unwrap();

        let local = match Player::local_player(self) { Some(p) => p, None => return };

        if cfg.flash_check   && local.is_flashed(self)   { return; }
        if cfg.visibility_check && !target.visible(self, &local) { return; }
        if !target.is_valid(self) { return; }
        if local.shots_fired(self) < cfg.start_bullet { return; }

        // best bone
        let (target_angle, _) = cfg.bones.iter()
            .map(|b| target.bone_position(self, b.u64()))
            .map(|p| self.angle_to_target(&local, &p, &self.target.previous_aim_punch))
            .map(|a| (a, angles_to_fov(&local.view_angles(self), &a)))
            .fold((Vec2::ZERO, 360.0), |best, cur| if cur.1 < best.1 { cur } else { best });

        let view = local.view_angles(self);
        let max_fov = cfg.fov * if cfg.distance_adjusted_fov {
            self.distance_scale(self.target.distance) } else { 1.0 };
        if angles_to_fov(&view, &target_angle) > max_fov { return; }

        // humanised move
        let mut aim = view - target_angle;
        if aim.y < -180.0 { aim.y += 360.0; }
        vec2_clamp(&mut aim);

        let sens = self.get_sensitivity() * local.fov_multiplier(self);
        let base = Vec2::new(aim.y / sens * 50.0, -aim.x / sens * 50.0);

        let tick = self.global_tick_count() as u32;
        let jitter = Vec2::new(white(tick), white(tick.wrapping_mul(7919))) * 0.35;
        let slip  = Vec2::new(white(tick.wrapping_mul(13001)), white(tick.wrapping_mul(19717))) * 0.20;

        let smooth = (cfg.smooth + 1.0).clamp(1.0, 20.0);
        let move_ = (base + jitter + slip) * human_curve(1.0 / smooth);

        // anti-twitch: skip 2 frames after big snap
        if self.aimbot_cooldown == 0 {
            mouse.move_rel(&move_);
            if move_.length_squared() > 0.02 { self.aimbot_cooldown = 2; }
        } else {
            self.aimbot_cooldown -= 1;
        }
    }
}
