use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

use glam::{IVec2, Mat4, Vec2, Vec3};
use x11rb::{
    connection::Connection as _,
    protocol::{
        Event,
        randr::{ConnectionExt as _, NotifyMask},
        xproto::{
            Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, Window,
        },
    },
    rust_connection::RustConnection,
};

use crate::{
    config::{
        Config,
        aim::{AimbotConfig, KeyMode, RcsConfig, TriggerbotConfig},
    },
    constants::cs2::{self, TEAM_CT, TEAM_T},
    cs2::{
        bones::Bones,
        entity::{
            Entity, EntityInfo, grenade_info, planted_c4::PlantedC4, player::Player, weapon::Weapon,
        },
        features::{aimbot::Aimbot, esp_toggle::EspToggle, rcs::Recoil, triggerbot::Triggerbot},
        input::Input,
        key_codes::KeyCode,
        offsets::Offsets,
        target::Target,
    },
    data::{Data, PlayerData},
    math::{angles_from_vector, vec2_clamp},
    os::{mouse::Mouse, process::Process},
    parser::{bvh::Bvh, read_map},
};

pub mod bones;
pub mod bvh;
pub mod entity;
mod features;
mod find_offsets;
mod input;
pub mod key_codes;
mod offsets;
mod schema;
mod target;

pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    input: Input,
    bvh: Option<Bvh>,
    current_bvh: String,
    target: Target,
    players: Vec<Player>,
    dead_players: Vec<Player>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    esp: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    gamescope: Option<Gamescope>,
    gamescope_geometry: Option<(Vec2, Vec2)>,
    last_cache: Instant,
}

impl CS2 {
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    pub fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        utils::info!("process found, pid: {}", process.pid);
        self.process = process;
        self.gamescope = find_gamescope(self.process.pid);
        self.gamescope_geometry = self.gamescope.as_ref().map(|gamescope| {
            gamescope
                .live_geometry()
                .unwrap_or_else(|| gamescope.fallback())
        });
        if let Some((position, size)) = self.gamescope_geometry {
            utils::info!(
                "detected gamescope geometry: {}x{} at {},{}",
                size.x,
                size.y,
                position.x,
                position.y
            );
        }

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        utils::info!("offsets found");

        self.is_valid = true;
    }

    pub fn run(&mut self, config: &Config, mouse: &mut Mouse) {
        if !self.process.is_valid() {
            self.is_valid = false;
            utils::debug!("process is no longer valid");
            return;
        }

        self.update_gamescope_geometry();

        self.input.update(&self.process, &self.offsets);

        if self.last_cache.elapsed() > Duration::from_millis(200) {
            self.cache_entities();
            self.check_bvh();
            self.last_cache = Instant::now();
        }

        for entity in &self.entities {
            if let Entity::Smoke(smoke) = entity {
                if config.misc.no_smoke {
                    smoke.disable(self);
                }

                if config.misc.change_smoke_color {
                    smoke.color(self, &config.misc.smoke_color);
                }
            }
        }

        self.no_flash(config);
        self.fov_changer(config);

        self.esp_toggle(config);

        self.triggerbot(config);

        self.triggerbot_shoot(mouse);

        self.find_target(config);

        if !self.aimbot(config, mouse) {
            self.rcs(config, mouse);
        }
    }

    pub fn data(&self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.spectators.clear();
        data.entities.clear();

        let sdl_window: usize = self.process.read(self.offsets.direct.sdl_window);
        if let Some((position, size)) = self.gamescope_geometry {
            data.window_position = position;
            data.window_size = size;
        } else if sdl_window == 0 {
            data.window_position = Vec2::ZERO;
            data.window_size = Vec2::ONE;
        } else {
            data.window_position = self.process.read::<IVec2>(sdl_window + 0x18).as_vec2();
            data.window_size = self
                .process
                .read::<IVec2>(sdl_window + 0x18 + 0x08)
                .as_vec2();
        }

        let Some(local_player) = Player::local_player(self) else {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        };
        let local_team = local_player.team(self);
        if local_team != TEAM_T && local_team != TEAM_CT {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        }
        let is_ffa = self.is_ffa();
        let spectator_target = local_player.spectator_target(self);
        let active_pawn = if let Some(target) = spectator_target {
            target.pawn
        } else {
            local_player.pawn
        };

        for player in &self.players {
            if spectator_target.is_some() && player.pawn == active_pawn {
                continue;
            }

            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head: player.bone_position(self, Bones::Head.u64()),
                name: player.name(self),
                weapon: player.weapon(self),
                ammo: (player.clip_ammo(self), player.reserve_ammo(self)),
                bones: player.all_bones(self),
                has_defuser: player.has_defuser(self),
                has_helmet: player.has_helmet(self),
                has_bomb: player.has_bomb(self),
                visible: player.visible(self, &local_player),
                color: player.color(self),
                rotation: player.rotation(self),
                sound: player.is_making_sound(self),
            };

            if !is_ffa && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        for player in &self.dead_players {
            if let Some(target) = player.spectator_target(self)
                && target.pawn == active_pawn
            {
                data.spectators.push(player.name(self));
            }
        }

        data.local_player = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_player.bone_position(self, Bones::Head.u64()),
            name: local_player.name(self),
            weapon: local_player.weapon(self),
            ammo: (
                local_player.clip_ammo(self),
                local_player.reserve_ammo(self),
            ),
            bones: local_player.all_bones(self),
            has_defuser: local_player.has_defuser(self),
            has_helmet: local_player.has_helmet(self),
            has_bomb: local_player.has_bomb(self),
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
            sound: None,
        };

        data.entities.clear();
        for entity in &self.entities {
            data.entities.push(match entity {
                Entity::Weapon { weapon, entity } => EntityInfo::Weapon {
                    weapon: weapon.clone(),
                    position: Player::entity(*entity).position(self),
                    ammo: (
                        Weapon::clip_ammo(*entity, self),
                        Weapon::reserve_ammo(*entity, self),
                    ),
                },
                Entity::Inferno(inferno) => EntityInfo::Inferno(inferno.info(self)),
                Entity::Smoke(smoke) => EntityInfo::Smoke(smoke.info(self)),
                Entity::Molotov(molotov) => EntityInfo::Molotov(molotov.info(self)),
                Entity::Flashbang(entity) => {
                    EntityInfo::Flashbang(grenade_info(*entity, "Flashbang", self))
                }
                Entity::HeGrenade(entity) => {
                    EntityInfo::HeGrenade(grenade_info(*entity, "HE Grenade", self))
                }
                Entity::Decoy(entity) => EntityInfo::Decoy(grenade_info(*entity, "Decoy", self)),
            });
        }

        data.weapon = local_player.weapon(self);
        data.in_game = true;
        data.is_ffa = is_ffa;
        data.map_name = self.current_map();
        data.aimbot_active = if self.aimbot_config(config).mode == KeyMode::Toggle {
            self.aim.active
        } else {
            false
        };
        data.triggerbot_active = if self.triggerbot_config(config).mode == KeyMode::Toggle {
            self.trigger.active
        } else {
            false
        };
        data.esp_active = self.esp_enabled(config);

        data.view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);
        data.view_angles = local_player.view_angles(self);

        if let Some(bomb) = &self.planted_c4 {
            data.bomb.planted = bomb.is_planted(self);
            data.bomb.timer = bomb.time_to_explosion(self);
            data.bomb.position = bomb.position(self);
            data.bomb.being_defused = bomb.is_being_defused(self);
            data.bomb.defuse_remain_time = bomb.time_to_defuse(self);
        } else {
            data.bomb.planted = false;
        }
    }

    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            input: Input::new(),
            bvh: None,
            current_bvh: String::new(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            dead_players: Vec::with_capacity(12),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            esp: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
            gamescope: None,
            gamescope_geometry: None,
            last_cache: Instant::now(),
        }
    }

    fn update_gamescope_geometry(&mut self) {
        let Some(geometry) = self.gamescope.as_mut().and_then(Gamescope::update_geometry) else {
            return;
        };
        if self.gamescope_geometry != Some(geometry) {
            let (position, size) = geometry;
            utils::debug!(
                "gamescope geometry changed: {}x{} at {},{}",
                size.x,
                size.y,
                position.x,
                position.y
            );
            self.gamescope_geometry = Some(geometry);
        }
    }

    fn aimbot_config<'a>(&self, config: &'a Config) -> &'a AimbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &config.aim.global.aimbot
    }

    fn rcs_config<'a>(&self, config: &'a Config) -> &'a RcsConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.rcs.enable_override
        {
            return &weapon_config.rcs;
        }
        &config.aim.global.rcs
    }

    fn triggerbot_config<'a>(&self, config: &'a Config) -> &'a TriggerbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.triggerbot.enable_override
        {
            return &weapon_config.triggerbot;
        }
        &config.aim.global.triggerbot
    }

    fn angle_to_target(&self, local_player: &Player, position: &Vec3, aim_punch: &Vec2) -> Vec2 {
        let eye_position = local_player.eye_position(self);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn entity_has_owner(&self, entity: usize) -> bool {
        self.process
            .read::<i32>(entity + self.offsets.controller.owner_entity)
            != -1
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x58)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u8>(self.offsets.convar.ffa + 0x58) == 1
    }

    fn current_time(&self) -> f32 {
        let global_vars: usize = self.process.read(self.offsets.direct.global_vars);
        self.process.read(global_vars + 0x30)
    }

    fn current_map(&self) -> String {
        let global_vars: usize = self.process.read(self.offsets.direct.global_vars);
        self.process
            .read_string(self.process.read(global_vars + 0x198))
    }

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }

    fn check_bvh(&mut self) {
        let current_map = self.current_map();
        if current_map != self.current_bvh {
            self.bvh = read_map(self);
            if self.bvh.is_some() {
                utils::info!("loaded bvh for {current_map}");
                self.current_bvh = current_map;
            }
        }
    }

    fn check_hotkey(input: &Input, mode: KeyMode, key: KeyCode, active: &mut bool) -> bool {
        match mode {
            KeyMode::Hold => input.is_key_pressed(key),
            KeyMode::Toggle => {
                if input.key_just_pressed(key) {
                    *active = !*active;
                }
                *active
            }
        }
    }
}

struct Gamescope {
    pid: i32,
    output_size: Vec2,
    x11: Option<GamescopeX11>,
}

impl Gamescope {
    fn new(pid: i32, output_size: Vec2) -> Self {
        Self {
            pid,
            output_size,
            x11: GamescopeX11::new(pid),
        }
    }

    fn live_geometry(&self) -> Option<(Vec2, Vec2)> {
        self.x11.as_ref()?.geometry(self.output_size)
    }

    fn update_geometry(&mut self) -> Option<(Vec2, Vec2)> {
        self.x11
            .as_mut()?
            .update_geometry(self.pid, self.output_size)
    }

    fn fallback(&self) -> (Vec2, Vec2) {
        (Vec2::ZERO, self.output_size)
    }
}

struct GamescopeX11 {
    connection: RustConnection,
    root: Window,
    client_list: Atom,
    pid_atom: Atom,
    window: Option<Window>,
}

impl GamescopeX11 {
    fn new(pid: i32) -> Option<Self> {
        let (connection, screen) = x11rb::connect(None).ok()?;
        let root = connection.setup().roots.get(screen)?.root;
        let client_list = connection
            .intern_atom(false, b"_NET_CLIENT_LIST")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let pid_atom = connection
            .intern_atom(false, b"_NET_WM_PID")
            .ok()?
            .reply()
            .ok()?
            .atom;

        connection
            .change_window_attributes(
                root,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
            )
            .ok()?;
        let _ = connection.randr_select_input(
            root,
            NotifyMask::SCREEN_CHANGE | NotifyMask::CRTC_CHANGE | NotifyMask::OUTPUT_CHANGE,
        );

        let mut x11 = Self {
            connection,
            root,
            client_list,
            pid_atom,
            window: None,
        };
        x11.track_window(pid);
        x11.connection.flush().ok()?;
        Some(x11)
    }

    fn update_geometry(&mut self, pid: i32, output_size: Vec2) -> Option<(Vec2, Vec2)> {
        let mut rediscover_window = false;
        let mut geometry_changed = false;

        loop {
            let event = match self.connection.poll_for_event() {
                Ok(Some(event)) => event,
                Ok(None) => break,
                Err(_) => return None,
            };

            match event {
                Event::ConfigureNotify(event) if Some(event.window) == self.window => {
                    geometry_changed = true;
                }
                Event::MapNotify(event) if Some(event.window) == self.window => {
                    geometry_changed = true;
                }
                Event::ReparentNotify(event) if Some(event.window) == self.window => {
                    geometry_changed = true;
                }
                Event::DestroyNotify(event) if Some(event.window) == self.window => {
                    self.window = None;
                    rediscover_window = true;
                    geometry_changed = true;
                }
                Event::UnmapNotify(event) if Some(event.window) == self.window => {
                    self.window = None;
                    rediscover_window = true;
                    geometry_changed = true;
                }
                Event::PropertyNotify(event)
                    if event.window == self.root && event.atom == self.client_list =>
                {
                    rediscover_window = true;
                    geometry_changed = true;
                }
                Event::RandrNotify(_) | Event::RandrScreenChangeNotify(_) => {
                    geometry_changed = true;
                }
                _ => {}
            }
        }

        if rediscover_window {
            self.track_window(pid);
        }

        if geometry_changed {
            self.geometry(output_size)
        } else {
            None
        }
    }

    fn track_window(&mut self, pid: i32) {
        let window = self.find_window(pid);
        if window == self.window {
            return;
        }
        self.window = window;

        if let Some(window) = window {
            let _ = self.connection.change_window_attributes(
                window,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
            );
            let _ = self.connection.flush();
        }
    }

    fn find_window(&self, pid: i32) -> Option<Window> {
        let windows = self
            .connection
            .get_property(
                false,
                self.root,
                self.client_list,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )
            .ok()?
            .reply()
            .ok()?;

        windows.value32()?.find(|window| {
            let Ok(cookie) = self.connection.get_property(
                false,
                *window,
                self.pid_atom,
                AtomEnum::CARDINAL,
                0,
                1,
            ) else {
                return false;
            };
            let Ok(reply) = cookie.reply() else {
                return false;
            };
            reply.value32().and_then(|mut values| values.next()) == Some(pid as u32)
        })
    }

    fn geometry(&self, output_size: Vec2) -> Option<(Vec2, Vec2)> {
        self.window
            .and_then(|window| self.window_geometry(window))
            .or_else(|| self.monitor_geometry(output_size))
    }

    fn window_geometry(&self, window: Window) -> Option<(Vec2, Vec2)> {
        let geometry = self.connection.get_geometry(window).ok()?.reply().ok()?;
        let position = self
            .connection
            .translate_coordinates(window, self.root, 0, 0)
            .ok()?
            .reply()
            .ok()?;
        Some((
            Vec2::new(position.dst_x as f32, position.dst_y as f32),
            Vec2::new(geometry.width as f32, geometry.height as f32),
        ))
    }

    fn monitor_geometry(&self, size: Vec2) -> Option<(Vec2, Vec2)> {
        let monitor = self
            .connection
            .randr_get_monitors(self.root, true)
            .ok()?
            .reply()
            .ok()?
            .monitors
            .into_iter()
            .find(|monitor| monitor.width as f32 == size.x && monitor.height as f32 == size.y)?;
        Some((
            Vec2::new(monitor.x as f32, monitor.y as f32),
            Vec2::new(monitor.width as f32, monitor.height as f32),
        ))
    }
}

fn find_gamescope(mut pid: i32) -> Option<Gamescope> {
    while pid > 1 {
        let raw = fs::read(format!("/proc/{pid}/cmdline")).ok()?;
        let args = raw
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg).into_owned())
            .collect::<Vec<_>>();

        if args
            .first()
            .and_then(|arg| Path::new(arg).file_name())
            .is_some_and(|name| name == "gamescope")
        {
            let (width, height) = parse_gamescope_output_size(&args);
            return Some(Gamescope::new(pid, Vec2::new(width as f32, height as f32)));
        }

        pid = fs::read_to_string(format!("/proc/{pid}/status"))
            .ok()?
            .lines()
            .find_map(|line| line.strip_prefix("PPid:"))?
            .trim()
            .parse()
            .ok()?;
    }

    None
}

fn parse_gamescope_output_size(args: &[String]) -> (u32, u32) {
    let args = &args[..args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len())];
    let value = |short: &str, long: &str| {
        args.iter().enumerate().find_map(|(index, arg)| {
            let value = if arg == short || arg == long {
                args.get(index + 1).map(String::as_str)
            } else {
                arg.strip_prefix(long)?.strip_prefix('=')
            };
            value?.parse::<u32>().ok().filter(|value| *value > 0)
        })
    };

    let output_height = value("-H", "--output-height");
    let height = output_height.unwrap_or(720);
    let width = value("-W", "--output-width").unwrap_or_else(|| {
        if output_height.is_some() {
            height.saturating_mul(16) / 9
        } else {
            1280
        }
    });
    (width, height)
}
