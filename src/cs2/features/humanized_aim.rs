use glam::{Vec2, vec2};

#[derive(Debug, Clone, Copy)]
pub struct HumanizedAimConfig {
    pub smooth: f32,
    pub inertia: f32,
    pub curve: f32,
    pub humanization: f32,
    pub settle_radius: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AimHumanizer {
    inertia: Vec2,
    phase: f32,
    curve_start_delta: Vec2,
    curve_start_distance: f32,
    curve_side: f32,
}

impl AimHumanizer {
    pub fn apply(&mut self, desired_delta: Vec2, config: HumanizedAimConfig) -> Vec2 {
        self.update_curve_path(desired_delta);

        let smooth = (config.smooth + 1.0).clamp(1.0, 20.0);
        let mut delta = desired_delta / smooth;

        let desired_length = desired_delta.length();
        if desired_length > 0.0 {
            self.phase += 0.37 + desired_length * 0.013;
            let direction = desired_delta / desired_length;
            let perpendicular = vec2(-direction.y, direction.x);
            let settle = if config.settle_radius > 0.0 {
                (desired_length / config.settle_radius).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let curve = self.curve_offset(desired_length, config.curve) / smooth * settle;
            let humanization =
                config.humanization.clamp(0.0, 1.0) * desired_length.min(12.0) * 0.08 / smooth
                    * settle;

            delta += perpendicular * curve;
            delta += vec2((self.phase * 1.7).sin(), (self.phase * 2.3).cos()) * humanization;
        }

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.inertia += (delta - self.inertia) * alpha;
        self.inertia
    }

    fn update_curve_path(&mut self, desired_delta: Vec2) {
        let distance = desired_delta.length();
        if distance <= 0.001 {
            self.curve_start_delta = Vec2::ZERO;
            self.curve_start_distance = 0.0;
            return;
        }

        let should_restart = self.curve_start_distance <= 0.001
            || distance > self.curve_start_distance * 1.15
            || self
                .curve_start_delta
                .normalize_or_zero()
                .dot(desired_delta.normalize_or_zero())
                < 0.65;

        if should_restart {
            self.curve_start_delta = desired_delta;
            self.curve_start_distance = distance;
            self.curve_side = if desired_delta.x * 0.37 + desired_delta.y * 0.19 >= 0.0 {
                1.0
            } else {
                -1.0
            };
        }
    }

    fn curve_offset(&self, distance: f32, curve: f32) -> f32 {
        if self.curve_start_distance <= 0.001 {
            return 0.0;
        }

        let strength = curve.clamp(0.0, 1.0);
        if strength <= 0.0 {
            return 0.0;
        }

        let progress = (1.0 - distance / self.curve_start_distance).clamp(0.0, 1.0);
        let lobes = if strength > 0.5 { 2.0 } else { 1.0 };
        let wave = (progress * std::f32::consts::PI * lobes).sin();
        let amplitude = self.curve_start_distance.min(80.0) * 0.12 * strength;
        wave * amplitude * self.curve_side
    }

    pub fn reset(&mut self) {
        self.inertia = Vec2::ZERO;
        self.phase = 0.0;
        self.curve_start_delta = Vec2::ZERO;
        self.curve_start_distance = 0.0;
        self.curve_side = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::{AimHumanizer, HumanizedAimConfig};

    #[test]
    fn inertia_eases_toward_desired_delta() {
        let mut humanizer = AimHumanizer::default();
        let config = HumanizedAimConfig {
            smooth: 0.0,
            inertia: 1.0,
            curve: 0.0,
            humanization: 0.0,
            settle_radius: 2.0,
        };

        let first = humanizer.apply(Vec2::new(10.0, 0.0), config);
        let second = humanizer.apply(Vec2::new(10.0, 0.0), config);

        assert!(first.x > 0.0);
        assert!(first.x < 10.0);
        assert!(second.x > first.x);
        assert!(second.x <= 10.0);
    }

    #[test]
    fn curve_adds_perpendicular_motion() {
        let mut humanizer = AimHumanizer::default();
        let config = HumanizedAimConfig {
            smooth: 0.0,
            inertia: 0.0,
            curve: 1.0,
            humanization: 0.0,
            settle_radius: 2.0,
        };

        let start = humanizer.apply(Vec2::new(10.0, 0.0), config);
        let shaped = humanizer.apply(Vec2::new(6.0, 0.0), config);

        assert!(start.y.abs() < 0.001);
        assert_ne!(shaped.y, 0.0);
    }

    #[test]
    fn curve_is_based_on_path_progress_not_ticks() {
        let mut humanizer = AimHumanizer::default();
        let config = HumanizedAimConfig {
            smooth: 0.0,
            inertia: 0.0,
            curve: 1.0,
            humanization: 0.0,
            settle_radius: 2.0,
        };

        let first = humanizer.apply(Vec2::new(10.0, 0.0), config);
        let curved = humanizer.apply(Vec2::new(6.0, 0.0), config);
        let repeated = humanizer.apply(Vec2::new(6.0, 0.0), config);

        assert!(first.y.abs() < 0.001);
        assert!(curved.y.abs() > 0.001);
        assert!((curved - repeated).length() < 0.001);
    }

    #[test]
    fn humanization_keeps_output_bounded() {
        let mut humanizer = AimHumanizer::default();
        let config = HumanizedAimConfig {
            smooth: 0.0,
            inertia: 0.0,
            curve: 0.0,
            humanization: 1.0,
            settle_radius: 2.0,
        };

        let shaped = humanizer.apply(Vec2::new(10.0, 0.0), config);

        assert!(shaped.length() > 0.0);
        assert!(shaped.length() < 13.0);
    }

    #[test]
    fn humanization_fades_out_near_target() {
        let mut humanized = AimHumanizer::default();
        let mut plain = AimHumanizer::default();
        let mut config = HumanizedAimConfig {
            smooth: 0.0,
            inertia: 0.0,
            curve: 1.0,
            humanization: 1.0,
            settle_radius: 2.0,
        };

        let near = Vec2::new(0.1, 0.0);
        let shaped = humanized.apply(near, config);
        config.curve = 0.0;
        config.humanization = 0.0;
        let unshaped = plain.apply(near, config);

        assert!((shaped - unshaped).length() < 0.02);
    }
}
