use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use glam::{Mat4, Quat, Vec2, Vec3};
use rand::{RngExt as _, rng};

use crate::{
    config::{Config, KeyMode, TriggerbotConfig},
    constants::cs2::{TEAM_CT, TEAM_T},
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    os::mouse::Mouse,
};

const TRIGGER_BASELINE_REACTION_MIN_MS: u32 = 60;
const TRIGGER_BASELINE_REACTION_MAX_MS: u32 = 110;
const TRIGGER_BASELINE_MIN_INTERVAL_MS: u32 = 80;
const TRIGGER_BASELINE_INTERVAL_JITTER_MS: u32 = 10;
const TRIGGER_BASELINE_HOLD_MIN_US: u32 = 25_000;
const TRIGGER_BASELINE_HOLD_MAX_US: u32 = 90_000;
const TAP_HOLD_MIN_US: u32 = 1_000;
const TAP_HOLD_MAX_US: u32 = 1_000_000;
const M_HMODEL: u64 = 160;
const PERM_MODEL_M_REFMESHES: u64 = 0x70;
const CRENDERMESH_HITBOX_DATA: u64 = 0x148;
const HITBOXSET_HITBOXES: u64 = 0x28;
const CHITBOX_STRIDE: u64 = 112;
const UTLVEC_SIZE: u64 = 0x00;
const UTLVEC_MEM: u64 = 0x08;
const HB_BONE_NAME: u64 = 16;
const HB_VMIN: u64 = 24;
const HB_VMAX: u64 = 36;
const MAX_HITBOXES: usize = 64;

#[derive(Debug)]
struct TriggerPending {
    scheduled_fire_at: Instant,
    chosen_hold_us: u32,
}

#[derive(Debug, Default)]
struct HitboxCache {
    by_model: HashMap<u64, Vec<HitboxDef>>,
}

#[derive(Debug, Clone)]
struct HitboxDef {
    bone_name: String,
    bone_index: Option<u64>,
    min_local: Vec3,
    max_local: Vec3,
}

#[derive(Debug, Clone, Copy)]
struct WorldCapsule {
    a: Vec3,
    b: Vec3,
}

#[derive(Debug, Clone, Copy)]
struct TriggerCandidate {
    player: Player,
    screen_distance: f32,
}

#[derive(Debug, Default)]
pub struct Triggerbot {
    pending_fire: Option<TriggerPending>,
    next_allowed_at: Option<Instant>,
    shot_end: Option<Instant>,
    hitbox_cache: HitboxCache,
    pub active: bool,
}

impl Triggerbot {
    fn reset_schedule(&mut self) {
        self.pending_fire = None;
        self.next_allowed_at = None;
    }

    fn process_trigger(
        &mut self,
        condition_met: bool,
        extra_reaction_delay_ms: u32,
        extra_min_interval_ms: u32,
        extra_hold_us: u32,
    ) -> Option<u32> {
        let now = Instant::now();

        if !condition_met {
            self.pending_fire = None;
            return None;
        }

        if let Some(blocked_until) = self.next_allowed_at
            && now < blocked_until
        {
            self.pending_fire = None;
            return None;
        }

        match self.pending_fire.take() {
            None => {
                let mut rng = rng();
                let baseline_reaction = rng.random_range(
                    TRIGGER_BASELINE_REACTION_MIN_MS..=TRIGGER_BASELINE_REACTION_MAX_MS,
                );
                let baseline_hold =
                    rng.random_range(TRIGGER_BASELINE_HOLD_MIN_US..=TRIGGER_BASELINE_HOLD_MAX_US);

                self.pending_fire = Some(TriggerPending {
                    scheduled_fire_at: now
                        + Duration::from_millis(
                            baseline_reaction.saturating_add(extra_reaction_delay_ms) as u64,
                        ),
                    chosen_hold_us: clamp_tap_hold_us(baseline_hold.saturating_add(extra_hold_us)),
                });
                None
            }
            Some(pending) if now >= pending.scheduled_fire_at => {
                let mut rng = rng();
                let lo = TRIGGER_BASELINE_MIN_INTERVAL_MS
                    .saturating_sub(TRIGGER_BASELINE_INTERVAL_JITTER_MS);
                let hi = TRIGGER_BASELINE_MIN_INTERVAL_MS + TRIGGER_BASELINE_INTERVAL_JITTER_MS;
                let baseline_interval = rng.random_range(lo..=hi);
                self.next_allowed_at = Some(
                    now + Duration::from_millis(
                        baseline_interval.saturating_add(extra_min_interval_ms) as u64,
                    ),
                );
                Some(pending.chosen_hold_us)
            }
            Some(pending) => {
                self.pending_fire = Some(pending);
                None
            }
        }
    }
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config, mouse: &mut Mouse) {
        let hotkey = config.aim.triggerbot_hotkey;
        let trigger_config = self.triggerbot_config(config).clone();

        if !trigger_config.enabled {
            self.trigger.reset_schedule();
            return;
        }

        let trigger_held = match trigger_config.mode {
            KeyMode::Hold => self.input.is_key_pressed(hotkey),
            KeyMode::Toggle => {
                if self.input.key_just_pressed(hotkey) {
                    self.trigger.active = !self.trigger.active;
                }
                self.trigger.active
            }
        };

        if !trigger_held {
            self.trigger.reset_schedule();
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.trigger.reset_schedule();
            return;
        };

        if !trigger_local_checks(self, &local_player, &trigger_config) {
            let _ = self.trigger.process_trigger(
                false,
                trigger_config.extra_reaction_delay_ms,
                trigger_config.extra_min_interval_ms,
                trigger_config.extra_hold_us,
            );
            return;
        }

        let mut hitbox_cache = std::mem::take(&mut self.trigger.hitbox_cache);
        let candidate =
            self.trigger_candidate_with_cache(&local_player, &trigger_config, &mut hitbox_cache);
        self.trigger.hitbox_cache = hitbox_cache;
        let crosshair_target = self.crosshair_trigger_target(&local_player, &trigger_config);

        if trigger_config.magnetized
            && let Some(target) = candidate
                .map(|candidate| candidate.player)
                .or(crosshair_target)
        {
            self.magnetized_triggerbot_aim_at(config, mouse, target);
        }

        if self.trigger.shot_end.is_some() {
            return;
        }

        let condition_met =
            trigger_condition_from_candidate(candidate, trigger_config.pixel_radius)
                || crosshair_target.is_some();
        if let Some(hold_us) = self.trigger.process_trigger(
            condition_met,
            trigger_config.extra_reaction_delay_ms,
            trigger_config.extra_min_interval_ms,
            trigger_config.extra_hold_us,
        ) {
            mouse.left_press();
            self.trigger.shot_end = Some(Instant::now() + Duration::from_micros(hold_us as u64));
        }
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        if let Some(shot_end) = self.trigger.shot_end
            && Instant::now() >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }
    }

    fn trigger_candidate_with_cache(
        &self,
        local_player: &Player,
        config: &TriggerbotConfig,
        hitbox_cache: &mut HitboxCache,
    ) -> Option<TriggerCandidate> {
        let screen_size = self.screen_size()?;
        let view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);
        let local_team = local_player.team(self);
        let team_check = config.team_check && !self.is_ffa();
        let mut best: Option<TriggerCandidate> = None;

        for player in &self.players {
            if !player.is_valid(self) {
                continue;
            }
            if team_check && player.team(self) == local_team {
                continue;
            }
            if config.visible_only && !player.visible(self, local_player) {
                continue;
            }

            let Some(distance) = self.closest_trigger_screen_distance(
                player,
                &view_matrix,
                screen_size,
                config,
                hitbox_cache,
            ) else {
                continue;
            };
            if best.is_none_or(|best| distance < best.screen_distance) {
                best = Some(TriggerCandidate {
                    player: *player,
                    screen_distance: distance,
                });
            }
        }

        best
    }

    fn crosshair_trigger_target(
        &self,
        local_player: &Player,
        config: &TriggerbotConfig,
    ) -> Option<Player> {
        let player = local_player.crosshair_entity(self)?;
        if !player.is_valid(self) {
            return None;
        }
        if config.team_check && !self.is_ffa() && player.team(self) == local_player.team(self) {
            return None;
        }
        Some(player)
    }

    fn closest_trigger_screen_distance(
        &self,
        player: &Player,
        view_matrix: &Mat4,
        screen_size: Vec2,
        config: &TriggerbotConfig,
        hitbox_cache: &mut HitboxCache,
    ) -> Option<f32> {
        let points = trigger_hitbox_points(player, self, config.head_only, hitbox_cache)
            .unwrap_or_else(|| trigger_skeleton_points(player, self, config.head_only));
        closest_projected_distance(points.iter().copied(), view_matrix, screen_size)
    }

    fn screen_size(&self) -> Option<Vec2> {
        let sdl_window = self.process.read::<u64>(self.offsets.direct.sdl_window);
        if sdl_window == 0 {
            return None;
        }

        let size = self
            .process
            .read::<glam::IVec2>(sdl_window + 0x18 + 0x08)
            .as_vec2();
        (size.x > 1.0 && size.y > 1.0).then_some(size)
    }
}

fn trigger_condition_from_candidate(
    candidate: Option<TriggerCandidate>,
    pixel_radius: f32,
) -> bool {
    trigger_condition_from_screen_distance(
        candidate.map(|candidate| candidate.screen_distance),
        pixel_radius,
    )
}

fn trigger_condition_from_screen_distance(distance: Option<f32>, pixel_radius: f32) -> bool {
    distance.is_some_and(|distance| distance <= pixel_radius.max(0.0))
}

fn trigger_local_checks(cs2: &CS2, local_player: &Player, config: &TriggerbotConfig) -> bool {
    let team = local_player.team(cs2);
    if team != TEAM_T && team != TEAM_CT {
        return false;
    }

    if config.flash_check && local_player.is_flashed(cs2) {
        return false;
    }

    if config.scope_check
        && local_player.weapon_class(cs2) == WeaponClass::Sniper
        && !local_player.is_scoped(cs2)
    {
        return false;
    }

    if config.velocity_check && local_player.velocity(cs2).length() > config.velocity_threshold {
        return false;
    }

    true
}

fn trigger_skeleton_points(player: &Player, cs2: &CS2, head_only: bool) -> Vec<Vec3> {
    if head_only {
        return vec![player.bone_position(cs2, Bones::Head.u64())];
    }

    let bones = player.all_bones(cs2);
    let mut points = Vec::with_capacity(Bones::CONNECTIONS.len() + 1);

    if let Some(head) = bones.get(&Bones::Head) {
        points.push(*head);
    }

    for (from, to) in Bones::CONNECTIONS {
        let (Some(from), Some(to)) = (bones.get(&from), bones.get(&to)) else {
            continue;
        };
        points.push((*from + *to) * 0.5);
    }

    points
}

fn trigger_hitbox_points(
    player: &Player,
    cs2: &CS2,
    head_only: bool,
    cache: &mut HitboxCache,
) -> Option<Vec<Vec3>> {
    let defs = hitbox_defs_for_player(player, cs2, cache)?;
    let mut points = Vec::with_capacity(defs.len());

    for hitbox in defs {
        if head_only && !hitbox.bone_name.to_ascii_lowercase().contains("head") {
            continue;
        }
        let Some(capsule) = world_capsule(player, cs2, hitbox) else {
            continue;
        };
        points.push((capsule.a + capsule.b) * 0.5);
    }

    (!points.is_empty()).then_some(points)
}

fn hitbox_defs_for_player<'a>(
    player: &Player,
    cs2: &CS2,
    cache: &'a mut HitboxCache,
) -> Option<&'a [HitboxDef]> {
    let model = player_model(player, cs2)?;
    if let std::collections::hash_map::Entry::Vacant(entry) = cache.by_model.entry(model) {
        let defs = read_hitbox_defs(cs2, model)?;
        entry.insert(defs);
    }
    cache.by_model.get(&model).map(Vec::as_slice)
}

fn player_model(player: &Player, cs2: &CS2) -> Option<u64> {
    let scene: u64 = cs2
        .process
        .read(player.pawn + cs2.offsets.pawn.game_scene_node);
    if !is_valid_ptr(scene) {
        return None;
    }
    let model_state = scene + cs2.offsets.game_scene_node.model_state;
    let handle: u64 = cs2.process.read(model_state + M_HMODEL);
    if !is_valid_ptr(handle) {
        return None;
    }
    let model: u64 = cs2.process.read(handle);
    is_valid_ptr(model).then_some(model)
}

fn read_hitbox_defs(cs2: &CS2, model: u64) -> Option<Vec<HitboxDef>> {
    let refmesh_vec = model + PERM_MODEL_M_REFMESHES;
    let refmesh_size: i32 = cs2.process.read(refmesh_vec + UTLVEC_SIZE);
    let refmesh_mem: u64 = cs2.process.read(refmesh_vec + UTLVEC_MEM);
    if refmesh_size <= 0 || !is_valid_ptr(refmesh_mem) {
        return None;
    }

    let render_mesh: u64 = cs2.process.read(refmesh_mem);
    if !is_valid_ptr(render_mesh) {
        return None;
    }

    let hitbox_data_vec = render_mesh + CRENDERMESH_HITBOX_DATA;
    let hitbox_data_size: i32 = cs2.process.read(hitbox_data_vec + UTLVEC_SIZE);
    let hitbox_data_mem: u64 = cs2.process.read(hitbox_data_vec + UTLVEC_MEM);
    if hitbox_data_size <= 0 || !is_valid_ptr(hitbox_data_mem) {
        return None;
    }

    let hitboxes_vec = hitbox_data_mem + HITBOXSET_HITBOXES;
    let hitboxes_size: i32 = cs2.process.read(hitboxes_vec + UTLVEC_SIZE);
    let hitboxes_mem: u64 = cs2.process.read(hitboxes_vec + UTLVEC_MEM);
    if hitboxes_size <= 0 || !is_valid_ptr(hitboxes_mem) {
        return None;
    }

    let count = (hitboxes_size as usize).min(MAX_HITBOXES);
    let mut defs = Vec::with_capacity(count);
    for index in 0..count {
        let hitbox = hitboxes_mem + index as u64 * CHITBOX_STRIDE;
        let bone_name = read_cstring_field(cs2, hitbox + HB_BONE_NAME).unwrap_or_default();
        if bone_name.is_empty() {
            continue;
        }

        defs.push(HitboxDef {
            bone_index: live_bone_index(&bone_name),
            bone_name,
            min_local: cs2.process.read(hitbox + HB_VMIN),
            max_local: cs2.process.read(hitbox + HB_VMAX),
        });
    }

    (!defs.is_empty()).then_some(defs)
}

fn world_capsule(player: &Player, cs2: &CS2, hitbox: &HitboxDef) -> Option<WorldCapsule> {
    let bone = bone_matrix(player, cs2, hitbox.bone_index?)?;
    Some(WorldCapsule {
        a: bone.transform_point3(hitbox.min_local),
        b: bone.transform_point3(hitbox.max_local),
    })
}

fn bone_matrix(player: &Player, cs2: &CS2, bone_index: u64) -> Option<Mat4> {
    let scene: u64 = cs2
        .process
        .read(player.pawn + cs2.offsets.pawn.game_scene_node);
    if !is_valid_ptr(scene) {
        return None;
    }
    let bone_data: u64 = cs2.process.read(
        scene + cs2.offsets.game_scene_node.model_state + cs2.offsets.model_state.skeleton_instance,
    );
    if !is_valid_ptr(bone_data) {
        return None;
    }

    let base = bone_data + bone_index * 32;
    let position: Vec3 = cs2.process.read(base);
    let scale: f32 = cs2.process.read(base + 12);
    let rotation = Quat::from_xyzw(
        cs2.process.read(base + 16),
        cs2.process.read(base + 20),
        cs2.process.read(base + 24),
        cs2.process.read(base + 28),
    );

    if !position.is_finite() || !scale.is_finite() || scale == 0.0 || !rotation.is_finite() {
        return None;
    }

    Some(Mat4::from_scale_rotation_translation(
        Vec3::splat(scale),
        rotation,
        position,
    ))
}

fn live_bone_index(bone_name: &str) -> Option<u64> {
    let bone = bone_name.to_ascii_lowercase();
    let stripped = strip_numeric_suffix(&bone);
    Some(
        match bone.as_str() {
            "pelvis" => Bones::Hip,
            "spine_0" => Bones::Spine1,
            "spine_1" => Bones::Spine2,
            "spine_2" => Bones::Spine3,
            "spine_3" => Bones::Spine4,
            "neck_0" | "neck" => Bones::Neck,
            "head_0" | "head" => Bones::Head,
            "clavicle_l" | "arm_upper_l" => Bones::LeftShoulder,
            "arm_lower_l" => Bones::LeftElbow,
            "hand_l" => Bones::LeftHand,
            "clavicle_r" | "arm_upper_r" => Bones::RightShoulder,
            "arm_lower_r" => Bones::RightElbow,
            "hand_r" => Bones::RightHand,
            "leg_upper_l" => Bones::LeftHip,
            "leg_lower_l" => Bones::LeftKnee,
            "ankle_l" | "foot_l" => Bones::LeftFoot,
            "leg_upper_r" => Bones::RightHip,
            "leg_lower_r" => Bones::RightKnee,
            "ankle_r" | "foot_r" => Bones::RightFoot,
            _ => match stripped {
                "neck" => Bones::Neck,
                "head" => Bones::Head,
                _ => return None,
            },
        }
        .u64(),
    )
}

fn strip_numeric_suffix(bone_name: &str) -> &str {
    if let Some(index) = bone_name.rfind('_') {
        let suffix = &bone_name[index + 1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return &bone_name[..index];
        }
    }
    bone_name
}

fn read_cstring_field(cs2: &CS2, address: u64) -> Option<String> {
    let pointer: u64 = cs2.process.read(address);
    if !is_valid_ptr(pointer) {
        return None;
    }
    Some(cs2.process.read_string(pointer))
}

fn is_valid_ptr(pointer: u64) -> bool {
    pointer > 0x10000 && pointer >> 48 == 0
}

fn closest_projected_distance(
    points: impl Iterator<Item = Vec3>,
    view_matrix: &Mat4,
    screen_size: Vec2,
) -> Option<f32> {
    let center = screen_size * 0.5;
    points
        .filter_map(|point| world_to_screen(point, view_matrix, screen_size))
        .map(|screen| screen.distance(center))
        .min_by(|left, right| left.total_cmp(right))
}

fn world_to_screen(position: Vec3, view_matrix: &Mat4, screen_size: Vec2) -> Option<Vec2> {
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

    if w < 0.0001 {
        return None;
    }

    screen_position /= w;
    screen_position.x = screen_size.x * 0.5 + 0.5 * screen_position.x * screen_size.x + 0.5;
    screen_position.y = screen_size.y * 0.5 - 0.5 * screen_position.y * screen_size.y + 0.5;

    if screen_position.x < 0.0
        || screen_position.x > screen_size.x
        || screen_position.y < 0.0
        || screen_position.y > screen_size.y
    {
        return None;
    }

    Some(screen_position)
}

fn clamp_tap_hold_us(hold_us: u32) -> u32 {
    hold_us.clamp(TAP_HOLD_MIN_US, TAP_HOLD_MAX_US)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn center_radius_condition_uses_projected_points() {
        let view_matrix = Mat4::IDENTITY;
        let screen_size = Vec2::new(100.0, 100.0);
        let points = [Vec3::new(0.0, 0.0, 1.0)];

        let distance =
            closest_projected_distance(points.into_iter(), &view_matrix, screen_size).unwrap();

        assert!(distance <= 2.5);
    }

    #[test]
    fn live_bone_index_matches_common_hitbox_names() {
        assert_eq!(live_bone_index("head_0"), Some(Bones::Head.u64()));
        assert_eq!(live_bone_index("spine_3"), Some(Bones::Spine4.u64()));
        assert_eq!(live_bone_index("arm_lower_L"), Some(Bones::LeftElbow.u64()));
        assert_eq!(live_bone_index("leg_lower_R"), Some(Bones::RightKnee.u64()));
        assert_eq!(live_bone_index("unknown"), None);
    }

    #[test]
    fn strip_numeric_suffix_only_removes_numeric_tail() {
        assert_eq!(strip_numeric_suffix("head_0"), "head");
        assert_eq!(strip_numeric_suffix("spine_3"), "spine");
        assert_eq!(strip_numeric_suffix("arm_lower_l"), "arm_lower_l");
    }

    #[test]
    fn center_radius_condition_rejects_far_projected_points() {
        let view_matrix = Mat4::IDENTITY;
        let screen_size = Vec2::new(100.0, 100.0);
        let points = [Vec3::new(0.5, 0.5, 1.0)];

        let distance =
            closest_projected_distance(points.into_iter(), &view_matrix, screen_size).unwrap();

        assert!(distance > 2.5);
    }

    #[test]
    fn trigger_candidate_uses_configured_radius() {
        assert!(trigger_condition_from_screen_distance(Some(2.0), 2.5));
        assert!(!trigger_condition_from_screen_distance(Some(3.0), 2.5));
        assert!(!trigger_condition_from_screen_distance(None, 2.5));
    }

    #[test]
    fn trigger_candidate_clamps_negative_radius() {
        assert!(trigger_condition_from_screen_distance(Some(0.0), -5.0));
        assert!(!trigger_condition_from_screen_distance(Some(0.1), -5.0));
    }

    #[test]
    fn trigger_schedules_then_fires_then_blocks() {
        let mut trigger = Triggerbot::default();
        assert_eq!(trigger.process_trigger(true, 0, 0, 0), None);

        sleep(Duration::from_millis(30));
        assert_eq!(trigger.process_trigger(true, 0, 0, 0), None);

        sleep(Duration::from_millis(130));
        assert!(trigger.process_trigger(true, 0, 0, 0).is_some());
        assert_eq!(trigger.process_trigger(true, 0, 0, 0), None);
    }

    #[test]
    fn trigger_hold_is_clamped_to_tap_bounds() {
        assert_eq!(clamp_tap_hold_us(0), TAP_HOLD_MIN_US);
        assert_eq!(clamp_tap_hold_us(500), TAP_HOLD_MIN_US);
        assert_eq!(clamp_tap_hold_us(25_000), 25_000);
        assert_eq!(clamp_tap_hold_us(u32::MAX), TAP_HOLD_MAX_US);
    }

    #[test]
    fn trigger_aborts_when_condition_drops() {
        let mut trigger = Triggerbot::default();
        assert_eq!(trigger.process_trigger(true, 0, 0, 0), None);
        assert!(trigger.pending_fire.is_some());

        assert_eq!(trigger.process_trigger(false, 0, 0, 0), None);

        assert!(trigger.pending_fire.is_none());
    }

    #[test]
    fn trigger_cooldown_dominates_pending_schedule() {
        let mut trigger = Triggerbot {
            pending_fire: Some(TriggerPending {
                scheduled_fire_at: Instant::now() - Duration::from_millis(1),
                chosen_hold_us: 25_000,
            }),
            next_allowed_at: Some(Instant::now() + Duration::from_millis(100)),
            ..Default::default()
        };

        assert_eq!(trigger.process_trigger(true, 0, 0, 0), None);
        assert!(trigger.pending_fire.is_none());
    }
}
