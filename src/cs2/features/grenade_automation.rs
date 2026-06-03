use glam::{Vec2, Vec3, vec2};

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::player::Player,
        features::humanized_aim::{AimHumanizer, HumanizedAimConfig},
        key_codes::KeyCode,
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::{InputKey, Mouse},
    ui::grenades::{Grenade, GrenadeList, ThrowButtons},
};

#[derive(Debug, Default)]
pub struct GrenadeAutomationState {
    active: Option<ActiveGrenadeAutomation>,
}

#[derive(Debug)]
struct ActiveGrenadeAutomation {
    target_position: Vec3,
    target_view_angles: Vec2,
    tick: u32,
    throw_start_tick: Option<u32>,
    movement_start_tick: Option<u32>,
    jump_start_tick: Option<u32>,
    throw_movement_keys: Vec<InputKey>,
    position_keys: Vec<InputKey>,
    aim_humanizer: AimHumanizer,
    buttons: ThrowButtons,
    positioned: bool,
    aimed: bool,
    should_jump: bool,
    jump_delay_ticks: u32,
    run_ticks: u32,
    throw_pressed: bool,
    jump_pressed: bool,
    movement_pressed: bool,
    finished: bool,
}

impl CS2 {
    pub fn grenade_automation(
        &mut self,
        config: &Config,
        grenades: &GrenadeList,
        mouse: &mut Mouse,
    ) -> bool {
        if self.grenade_automation.active.is_some() {
            if !self.input.is_key_pressed(config.grenade.automation_hotkey) {
                self.grenade_automation
                    .active
                    .as_mut()
                    .unwrap()
                    .cancel(mouse);
                self.grenade_automation.active = None;
                return false;
            }
            let Some(local_player) = Player::local_player(self) else {
                self.grenade_automation
                    .active
                    .as_mut()
                    .unwrap()
                    .cancel(mouse);
                self.grenade_automation.active = None;
                return false;
            };
            let view_angles = local_player.view_angles(self);
            let position = local_player.position(self);
            let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
            let grenade_config = config.grenade.clone();
            let blocked_keys = self
                .grenade_automation
                .active
                .as_ref()
                .unwrap()
                .blocked_input_keys(config);
            self.input
                .clear_game_keys(&self.process, &self.offsets, &blocked_keys);
            let active = self.grenade_automation.active.as_mut().unwrap();
            active.reassert_inputs(mouse);
            let finished =
                active.update(mouse, position, view_angles, sensitivity, &grenade_config);
            if finished {
                self.grenade_automation.active = None;
            }
            return true;
        }

        if !config.grenade.automation_enabled {
            return false;
        }
        if !self
            .input
            .key_just_pressed(config.grenade.automation_hotkey)
        {
            return false;
        }

        let Some(grenade) = self.active_grenade(config, grenades) else {
            return false;
        };

        let active = ActiveGrenadeAutomation::new(grenade);
        self.grenade_automation.active = Some(active);
        true
    }

    fn active_grenade<'a>(
        &self,
        config: &Config,
        grenades: &'a GrenadeList,
    ) -> Option<&'a Grenade> {
        let map = self.current_map();
        let local_player = Player::local_player(self)?;
        let position = local_player.position(self);
        let view_angles = local_player.view_angles(self);
        let weapon = local_player.weapon(self);
        let activation_distance = config.grenade.marker_draw_distance;

        let candidates = grenades
            .get(&map)?
            .iter()
            .filter(|grenade| grenade.weapon == weapon)
            .filter(|grenade| (position - grenade.position).length() <= activation_distance);

        choose_grenade_by_position_then_crosshair(
            candidates,
            position,
            view_angles,
            config.grenade.activation_distance,
            config.grenade.aim_fov,
        )
    }
}

impl ActiveGrenadeAutomation {
    fn new(grenade: &Grenade) -> Self {
        let throw_movement_keys = movement_keys(grenade);
        let buttons = grenade.automation.throw_buttons(&grenade.description);
        let should_jump = grenade.modifiers.jump
            || grenade.automation.movement_flags & 4 != 0
            || grenade.description.to_ascii_lowercase().contains("jump");

        Self {
            target_position: grenade.position,
            target_view_angles: grenade.view_angles,
            tick: 0,
            throw_start_tick: None,
            movement_start_tick: None,
            jump_start_tick: None,
            throw_movement_keys,
            position_keys: Vec::new(),
            aim_humanizer: AimHumanizer::default(),
            buttons,
            positioned: false,
            aimed: false,
            should_jump,
            jump_delay_ticks: grenade.automation.jump_delay_ticks,
            run_ticks: grenade.automation.run_ticks,
            throw_pressed: false,
            jump_pressed: false,
            movement_pressed: false,
            finished: false,
        }
    }

    fn press_throw_movement(&mut self, mouse: &mut Mouse) {
        for key in &self.throw_movement_keys {
            mouse.press_key(*key);
        }
    }

    fn reassert_inputs(&mut self, mouse: &mut Mouse) {
        if self.throw_pressed {
            self.press_throw(mouse);
        }
        if self.movement_pressed {
            self.press_throw_movement(mouse);
        }
        for key in &self.position_keys {
            mouse.press_key(*key);
        }
        if self.jump_pressed {
            mouse.press_key(InputKey::Space);
        }
    }

    fn blocked_input_keys(&self, config: &Config) -> Vec<KeyCode> {
        const BLOCKED_KEYS: &[KeyCode] = &[
            KeyCode::W,
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::Space,
            KeyCode::LeftShift,
            KeyCode::LeftControl,
            KeyCode::MouseLeft,
            KeyCode::MouseRight,
        ];

        BLOCKED_KEYS
            .iter()
            .copied()
            .filter(|key| *key != config.grenade.automation_hotkey)
            .filter(|key| !self.owned_input_keys().contains(key))
            .collect()
    }

    fn owned_input_keys(&self) -> Vec<KeyCode> {
        let mut keys = Vec::new();

        if self.throw_pressed {
            keys.extend(throw_button_key_codes(self.buttons));
        }
        if self.movement_pressed {
            keys.extend(self.throw_movement_keys.iter().map(|key| key_code(*key)));
        }
        keys.extend(self.position_keys.iter().map(|key| key_code(*key)));
        if self.jump_pressed {
            keys.push(KeyCode::Space);
        }

        keys
    }

    fn update(
        &mut self,
        mouse: &mut Mouse,
        position: Vec3,
        view_angles: Vec2,
        sensitivity: f32,
        config: &crate::config::GrenadeConfig,
    ) -> bool {
        if self.finished {
            return true;
        }

        if !self.positioned {
            let position_keys = movement_keys_to_position(
                position,
                self.target_position,
                view_angles,
                config.position_tolerance,
            );
            if !position_keys.is_empty() {
                self.update_position_keys(mouse, position_keys);
                return false;
            }
            self.release_position_keys(mouse);
            self.positioned = true;
        }

        if !self.aimed {
            if should_adjust_aim(
                self.aimed,
                view_angles,
                self.target_view_angles,
                config.aim_tolerance,
            ) {
                mouse.move_rel(mouse_delta_to_target(
                    &mut self.aim_humanizer,
                    view_angles,
                    self.target_view_angles,
                    sensitivity,
                    HumanizedAimConfig {
                        smooth: config.aim_smooth,
                        inertia: config.aim_inertia,
                        curve: config.aim_curve,
                        humanization: config.aim_humanization,
                        settle_radius: config.aim_tolerance.max(0.01) * 45.45,
                    },
                ));
                return false;
            }
            self.aimed = true;
        }

        if !self.throw_pressed {
            self.press_throw(mouse);
            self.throw_start_tick = Some(self.tick);
            self.tick += 1;
            return false;
        }

        if !self.movement_pressed {
            if !self.can_start_throw_movement() {
                self.tick += 1;
                return false;
            }
            self.press_throw_movement(mouse);
            self.movement_pressed = true;
            self.movement_start_tick = Some(self.tick);
        }

        let movement_start_tick = self.movement_start_tick.unwrap_or(self.tick);

        if self.should_jump
            && self.throw_pressed
            && !self.jump_pressed
            && self.tick >= movement_start_tick + self.run_ticks
        {
            mouse.press_key(InputKey::Space);
            self.jump_pressed = true;
            self.jump_start_tick = Some(self.tick);
        }

        if self.throw_pressed && self.should_release_throw() {
            self.release_throw(mouse);
            if self.jump_pressed {
                mouse.release_key(InputKey::Space);
                self.jump_pressed = false;
            }
            for key in self.throw_movement_keys.iter().rev() {
                mouse.release_key(*key);
            }
            self.release_position_keys(mouse);
            self.finished = true;
            return true;
        }

        self.tick += 1;
        false
    }

    fn can_start_throw_movement(&self) -> bool {
        self.throw_pressed
            && self
                .throw_start_tick
                .is_some_and(|throw_start_tick| self.tick > throw_start_tick)
    }

    fn should_release_throw(&self) -> bool {
        if self.should_jump {
            let Some(jump_start_tick) = self.jump_start_tick else {
                return false;
            };
            return self.tick >= jump_start_tick + self.jump_delay_ticks.max(1);
        }

        let Some(throw_start_tick) = self.throw_start_tick else {
            return false;
        };
        let Some(movement_start_tick) = self.movement_start_tick else {
            return false;
        };
        self.tick >= throw_start_tick + MIN_THROW_HOLD_TICKS
            && self.tick > movement_start_tick + self.run_ticks
    }

    fn press_throw(&mut self, mouse: &mut Mouse) {
        match self.buttons {
            ThrowButtons::Left => mouse.left_press(),
            ThrowButtons::Right => mouse.right_press(),
            ThrowButtons::Both => {
                mouse.right_press();
                mouse.left_press();
            }
        }
        self.throw_pressed = true;
    }

    fn release_throw(&mut self, mouse: &mut Mouse) {
        match self.buttons {
            ThrowButtons::Left => mouse.left_release(),
            ThrowButtons::Right => mouse.right_release(),
            ThrowButtons::Both => {
                mouse.left_release();
                mouse.right_release();
            }
        }
        self.throw_pressed = false;
    }

    fn update_position_keys(&mut self, mouse: &mut Mouse, next_keys: Vec<InputKey>) {
        for key in &self.position_keys {
            if !next_keys.contains(key) {
                mouse.release_key(*key);
            }
        }
        for key in &next_keys {
            if !self.position_keys.contains(key) {
                mouse.press_key(*key);
            }
        }
        self.position_keys = next_keys;
    }

    fn release_position_keys(&mut self, mouse: &mut Mouse) {
        for key in self.position_keys.iter().rev() {
            mouse.release_key(*key);
        }
        self.position_keys.clear();
    }

    fn cancel(&mut self, mouse: &mut Mouse) {
        if self.throw_pressed {
            self.release_throw(mouse);
        }
        if self.jump_pressed {
            mouse.release_key(InputKey::Space);
            self.jump_pressed = false;
        }
        for key in self.throw_movement_keys.iter().rev() {
            mouse.release_key(*key);
        }
        self.movement_pressed = false;
        self.release_position_keys(mouse);
        self.finished = true;
    }
}

const MIN_THROW_HOLD_TICKS: u32 = 2;

fn choose_grenade_by_crosshair<'a>(
    grenades: impl Iterator<Item = &'a Grenade>,
    view_angles: Vec2,
) -> Option<&'a Grenade> {
    grenades.min_by(|left, right| {
        angles_to_fov(&view_angles, &left.view_angles)
            .total_cmp(&angles_to_fov(&view_angles, &right.view_angles))
    })
}

fn choose_grenade_by_position_then_crosshair<'a>(
    grenades: impl Iterator<Item = &'a Grenade>,
    position: Vec3,
    view_angles: Vec2,
    same_spot_distance: f32,
    max_aim_fov: f32,
) -> Option<&'a Grenade> {
    let grenades: Vec<_> = grenades
        .filter(|grenade| angles_to_fov(&view_angles, &grenade.view_angles) <= max_aim_fov)
        .collect();
    let closest_distance = grenades
        .iter()
        .map(|grenade| (position - grenade.position).length())
        .min_by(|left, right| left.total_cmp(right))?;
    let same_spot_distance = same_spot_distance.max(1.0);

    choose_grenade_by_crosshair(
        grenades.into_iter().filter(|grenade| {
            (position - grenade.position).length() <= closest_distance + same_spot_distance
        }),
        view_angles,
    )
}

fn is_lined_up(view_angles: Vec2, target_view_angles: Vec2, tolerance: f32) -> bool {
    angles_to_fov(&view_angles, &target_view_angles) <= tolerance.max(0.01)
}

fn should_adjust_aim(
    aimed: bool,
    view_angles: Vec2,
    target_view_angles: Vec2,
    tolerance: f32,
) -> bool {
    !aimed && !is_lined_up(view_angles, target_view_angles, tolerance)
}

fn mouse_delta_to_target(
    humanizer: &mut AimHumanizer,
    view_angles: Vec2,
    target_view_angles: Vec2,
    sensitivity: f32,
    config: HumanizedAimConfig,
) -> Vec2 {
    let mut aim_angles = view_angles - target_view_angles;
    if aim_angles.y < -180.0 {
        aim_angles.y += 360.0;
    }
    vec2_clamp(&mut aim_angles);

    let mouse_delta = vec2(
        aim_angles.y / sensitivity * 45.45,
        -aim_angles.x / sensitivity * 45.45,
    );

    let mut mouse_delta = humanizer.apply(mouse_delta, config);

    if mouse_delta.x != 0.0 && mouse_delta.x.abs() < 1.0 {
        mouse_delta.x = mouse_delta.x.signum();
    }
    if mouse_delta.y != 0.0 && mouse_delta.y.abs() < 1.0 {
        mouse_delta.y = mouse_delta.y.signum();
    }

    mouse_delta
}

fn movement_keys_to_position(
    position: Vec3,
    target_position: Vec3,
    view_angles: Vec2,
    tolerance: f32,
) -> Vec<InputKey> {
    let delta = target_position - position;
    if delta.truncate().length() <= tolerance.max(0.5) {
        return Vec::new();
    }

    let yaw = view_angles.y.to_radians();
    let forward = Vec2::new(yaw.cos(), yaw.sin());
    let right = Vec2::new(yaw.sin(), -yaw.cos());
    let flat_delta = delta.truncate();
    let forward_amount = flat_delta.dot(forward);
    let right_amount = flat_delta.dot(right);
    let axis_tolerance = tolerance.max(0.5);
    let mut keys = Vec::with_capacity(2);

    if forward_amount > axis_tolerance {
        keys.push(InputKey::W);
    } else if forward_amount < -axis_tolerance {
        keys.push(InputKey::S);
    }

    if right_amount > axis_tolerance {
        keys.push(InputKey::D);
    } else if right_amount < -axis_tolerance {
        keys.push(InputKey::A);
    }

    keys
}

fn movement_keys(grenade: &Grenade) -> Vec<InputKey> {
    let mut keys = Vec::with_capacity(5);
    let description = grenade.description.to_ascii_lowercase();
    let flags = grenade.automation.movement_flags;

    if grenade.modifiers.duck || flags & 1 != 0 || description.contains("crouch") {
        keys.push(InputKey::Ctrl);
    }
    if flags & 8 != 0 || description.contains("walk") {
        keys.push(InputKey::Shift);
    }

    let forward = grenade.modifiers.run
        || grenade.automation.always_run
        || flags & 2 != 0
        || flags & 16 != 0
        || flags & 512 != 0
        || description.contains("w+")
        || description.contains("run")
        || description.contains("walk")
        || description.contains("step");
    if forward {
        keys.push(InputKey::W);
    }

    keys
}

fn key_code(key: InputKey) -> KeyCode {
    match key {
        InputKey::W => KeyCode::W,
        InputKey::A => KeyCode::A,
        InputKey::S => KeyCode::S,
        InputKey::D => KeyCode::D,
        InputKey::Space => KeyCode::Space,
        InputKey::Ctrl => KeyCode::LeftControl,
        InputKey::Shift => KeyCode::LeftShift,
        InputKey::MouseLeft => KeyCode::MouseLeft,
        InputKey::MouseRight => KeyCode::MouseRight,
    }
}

fn throw_button_key_codes(buttons: ThrowButtons) -> Vec<KeyCode> {
    match buttons {
        ThrowButtons::Left => vec![KeyCode::MouseLeft],
        ThrowButtons::Right => vec![KeyCode::MouseRight],
        ThrowButtons::Both => vec![KeyCode::MouseRight, KeyCode::MouseLeft],
    }
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, Vec3};
    use uuid::Uuid;

    use super::{
        ActiveGrenadeAutomation, MIN_THROW_HOLD_TICKS, choose_grenade_by_crosshair,
        choose_grenade_by_position_then_crosshair, is_lined_up, mouse_delta_to_target,
        movement_keys, movement_keys_to_position, should_adjust_aim,
    };
    use crate::{
        config::Config,
        cs2::{
            entity::weapon::Weapon,
            features::humanized_aim::{AimHumanizer, HumanizedAimConfig},
            key_codes::KeyCode,
        },
        os::mouse::InputKey,
        ui::grenades::{Grenade, GrenadeAutomation, GrenadeModifiers, ThrowButtons},
    };

    fn grenade(description: &str, flags: u32, modifiers: GrenadeModifiers) -> Grenade {
        Grenade {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            description: description.to_string(),
            position: Vec3::ZERO,
            view_angles: Vec2::ZERO,
            weapon: Weapon::Smoke,
            modifiers,
            automation: GrenadeAutomation {
                movement_flags: flags,
                ..Default::default()
            },
        }
    }

    fn aim_config(smooth: f32) -> HumanizedAimConfig {
        HumanizedAimConfig {
            smooth,
            inertia: 0.0,
            curve: 0.0,
            humanization: 0.0,
            settle_radius: 2.0,
        }
    }

    #[test]
    fn movement_keys_keep_directional_instructions() {
        let keys = movement_keys(&grenade("W+Jump+Throw", 0, GrenadeModifiers::default()));

        assert!(keys.contains(&InputKey::W));
    }

    #[test]
    fn movement_flags_keep_duck_forward_run_bits() {
        let keys = movement_keys(&grenade("Run+Jump+Throw", 1 | 2 | 16, Default::default()));

        assert_eq!(keys, vec![InputKey::Ctrl, InputKey::W]);
    }

    #[test]
    fn always_run_and_run_flag_keep_forward_movement() {
        let mut grenade = grenade("Jump+Throw", 512 | 4, Default::default());
        grenade.automation.always_run = true;

        let keys = movement_keys(&grenade);

        assert!(keys.contains(&InputKey::W));
    }

    #[test]
    fn run_jump_flags_do_not_create_strafe_movement() {
        let keys = movement_keys(&grenade("Run+Jump+Throw", 534, Default::default()));

        assert_eq!(keys, vec![InputKey::W]);
    }

    #[test]
    fn walk_jump_flags_hold_shift_and_forward() {
        let keys = movement_keys(&grenade("Walk+Jump+Throw", 526, Default::default()));

        assert_eq!(keys, vec![InputKey::Shift, InputKey::W]);
    }

    #[test]
    fn input_blocking_keeps_hotkey_visible() {
        let mut config = Config::default();
        config.grenade.automation_hotkey = KeyCode::MouseRight;
        let active = ActiveGrenadeAutomation::new(&grenade("", 0, Default::default()));

        let blocked = active.blocked_input_keys(&config);

        assert!(!blocked.contains(&KeyCode::MouseRight));
    }

    #[test]
    fn input_blocking_keeps_active_position_movement() {
        let mut active = ActiveGrenadeAutomation::new(&grenade("", 0, Default::default()));
        active.position_keys = vec![InputKey::W, InputKey::D];

        let blocked = active.blocked_input_keys(&Config::default());

        assert!(!blocked.contains(&KeyCode::W));
        assert!(!blocked.contains(&KeyCode::D));
        assert!(blocked.contains(&KeyCode::A));
        assert!(blocked.contains(&KeyCode::S));
    }

    #[test]
    fn input_blocking_keeps_active_throw_buttons() {
        let mut active = ActiveGrenadeAutomation::new(&grenade("", 0, Default::default()));
        active.buttons = ThrowButtons::Both;
        active.throw_pressed = true;

        let blocked = active.blocked_input_keys(&Config::default());

        assert!(!blocked.contains(&KeyCode::MouseLeft));
        assert!(!blocked.contains(&KeyCode::MouseRight));
    }

    #[test]
    fn input_blocking_keeps_active_throw_movement_and_jump() {
        let mut active = ActiveGrenadeAutomation::new(&grenade("", 0, Default::default()));
        active.throw_movement_keys = vec![InputKey::Ctrl, InputKey::Shift, InputKey::W];
        active.movement_pressed = true;
        active.jump_pressed = true;

        let blocked = active.blocked_input_keys(&Config::default());

        assert!(!blocked.contains(&KeyCode::LeftControl));
        assert!(!blocked.contains(&KeyCode::LeftShift));
        assert!(!blocked.contains(&KeyCode::W));
        assert!(!blocked.contains(&KeyCode::Space));
    }

    #[test]
    fn movement_keys_to_position_use_current_yaw() {
        let keys = movement_keys_to_position(
            Vec3::ZERO,
            Vec3::new(100.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
            2.0,
        );
        assert_eq!(keys, vec![InputKey::W]);

        let keys = movement_keys_to_position(
            Vec3::ZERO,
            Vec3::new(100.0, 0.0, 0.0),
            Vec2::new(0.0, 90.0),
            2.0,
        );
        assert_eq!(keys, vec![InputKey::D]);
    }

    #[test]
    fn movement_keys_to_position_stop_inside_tolerance() {
        let keys = movement_keys_to_position(
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec2::new(0.0, 0.0),
            2.0,
        );
        assert!(keys.is_empty());
    }

    #[test]
    fn choose_grenade_by_crosshair_prefers_nearest_aimpoint() {
        let mut left = grenade("", 0, Default::default());
        left.name = "left".to_string();
        left.view_angles = Vec2::new(0.0, -20.0);

        let mut center = grenade("", 0, Default::default());
        center.name = "center".to_string();
        center.view_angles = Vec2::new(0.0, 2.0);

        let chosen =
            choose_grenade_by_crosshair([&left, &center].into_iter(), Vec2::new(0.0, 0.0)).unwrap();

        assert_eq!(chosen.name, "center");
    }

    #[test]
    fn choose_grenade_prefers_closest_position_before_crosshair() {
        let mut close = grenade("", 0, Default::default());
        close.name = "close".to_string();
        close.position = Vec3::new(8.0, 0.0, 0.0);
        close.view_angles = Vec2::new(0.0, 90.0);

        let mut far_crosshair = grenade("", 0, Default::default());
        far_crosshair.name = "far_crosshair".to_string();
        far_crosshair.position = Vec3::new(200.0, 0.0, 0.0);
        far_crosshair.view_angles = Vec2::new(0.0, 0.0);

        let chosen = choose_grenade_by_position_then_crosshair(
            [&close, &far_crosshair].into_iter(),
            Vec3::ZERO,
            Vec2::ZERO,
            24.0,
            180.0,
        )
        .unwrap();

        assert_eq!(chosen.name, "close");
    }

    #[test]
    fn choose_grenade_uses_crosshair_within_closest_position_cluster() {
        let mut left = grenade("", 0, Default::default());
        left.name = "left".to_string();
        left.position = Vec3::new(8.0, 0.0, 0.0);
        left.view_angles = Vec2::new(0.0, -45.0);

        let mut center = grenade("", 0, Default::default());
        center.name = "center".to_string();
        center.position = Vec3::new(10.0, 0.0, 0.0);
        center.view_angles = Vec2::new(0.0, 1.0);

        let chosen = choose_grenade_by_position_then_crosshair(
            [&left, &center].into_iter(),
            Vec3::ZERO,
            Vec2::ZERO,
            24.0,
            180.0,
        )
        .unwrap();

        assert_eq!(chosen.name, "center");
    }

    #[test]
    fn choose_grenade_ignores_aimpoints_outside_fov() {
        let mut behind = grenade("", 0, Default::default());
        behind.name = "behind".to_string();
        behind.position = Vec3::new(8.0, 0.0, 0.0);
        behind.view_angles = Vec2::new(0.0, 170.0);

        let mut visible = grenade("", 0, Default::default());
        visible.name = "visible".to_string();
        visible.position = Vec3::new(12.0, 0.0, 0.0);
        visible.view_angles = Vec2::new(0.0, 10.0);

        let chosen = choose_grenade_by_position_then_crosshair(
            [&behind, &visible].into_iter(),
            Vec3::ZERO,
            Vec2::ZERO,
            24.0,
            25.0,
        )
        .unwrap();

        assert_eq!(chosen.name, "visible");
    }

    #[test]
    fn lineup_check_handles_yaw_wraparound() {
        assert!(is_lined_up(
            Vec2::new(0.0, 179.95),
            Vec2::new(0.0, -179.95),
            0.2,
        ));
        assert!(!is_lined_up(
            Vec2::new(0.0, 170.0),
            Vec2::new(0.0, -170.0),
            0.2,
        ));
    }

    #[test]
    fn mouse_delta_to_target_is_smoothed() {
        let snapped = mouse_delta_to_target(
            &mut AimHumanizer::default(),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 20.0),
            2.0,
            aim_config(0.0),
        );
        let smoothed = mouse_delta_to_target(
            &mut AimHumanizer::default(),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 20.0),
            2.0,
            aim_config(9.0),
        );

        assert!(smoothed.length() < snapped.length());
        assert!(smoothed.length() > 0.0);
    }

    #[test]
    fn mouse_delta_to_target_keeps_integer_nudge_near_alignment() {
        let delta = mouse_delta_to_target(
            &mut AimHumanizer::default(),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.25),
            2.0,
            aim_config(19.0),
        );

        assert!(delta.x.abs() >= 1.0);
    }

    #[test]
    fn jump_throw_waits_until_after_attack_settles() {
        let mut active = ActiveGrenadeAutomation::new(&grenade(
            "Jump+Throw LRMB",
            4,
            GrenadeModifiers {
                jump: true,
                ..Default::default()
            },
        ));

        active.tick = 0;
        active.throw_pressed = true;
        active.throw_start_tick = Some(0);
        assert!(!active.should_release_throw());

        active.tick = 1;
        active.jump_pressed = true;
        active.jump_start_tick = Some(1);
        assert!(!active.should_release_throw());

        active.tick = 2;
        assert!(active.should_release_throw());
    }

    #[test]
    fn non_jump_throw_holds_attack_for_multiple_ticks() {
        let mut active =
            ActiveGrenadeAutomation::new(&grenade("Throw LRMB", 0, GrenadeModifiers::default()));

        active.throw_pressed = true;
        active.throw_start_tick = Some(0);
        active.movement_start_tick = Some(1);
        active.tick = 1;
        assert!(!active.should_release_throw());

        active.tick = MIN_THROW_HOLD_TICKS;
        assert!(active.should_release_throw());
    }

    #[test]
    fn throw_movement_waits_until_after_prime_tick() {
        let mut active = ActiveGrenadeAutomation::new(&grenade(
            "W+Jump+Throw",
            518,
            GrenadeModifiers {
                jump: true,
                run: true,
                ..Default::default()
            },
        ));

        active.tick = 0;
        active.throw_pressed = true;
        active.throw_start_tick = Some(0);
        assert!(!active.can_start_throw_movement());

        active.tick = 1;
        assert!(active.can_start_throw_movement());
    }

    #[test]
    fn aim_phase_latches_before_throw_sequence() {
        assert!(!should_adjust_aim(
            false,
            Vec2::ZERO,
            Vec2::new(0.0, 0.1),
            0.2,
        ));
        assert!(should_adjust_aim(
            false,
            Vec2::ZERO,
            Vec2::new(0.0, 10.0),
            0.2,
        ));
        assert!(!should_adjust_aim(
            true,
            Vec2::ZERO,
            Vec2::new(0.0, 10.0),
            0.2,
        ));
    }

    #[test]
    fn positioning_phase_latches_after_centering() {
        let mut active = ActiveGrenadeAutomation::new(&grenade(
            "W+Jump+Throw",
            518,
            GrenadeModifiers {
                run: true,
                ..Default::default()
            },
        ));

        assert!(!active.positioned);
        active.positioned = true;

        let keys = movement_keys_to_position(
            Vec3::new(128.0, 0.0, 0.0),
            active.target_position,
            Vec2::ZERO,
            2.0,
        );
        assert!(!keys.is_empty());
        assert!(active.positioned);
    }
}
