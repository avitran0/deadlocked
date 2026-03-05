use std::{
    collections::HashMap,
    fs::read_to_string,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use utils::log;

use crate::{
    cs2::{bones::Bones, entity::weapon::Weapon, key_codes::KeyCode},
    ui::color::Colors,
};

const REFRESH_RATE: u64 = 100;
pub const LOOP_DURATION: Duration = Duration::from_millis(1000 / REFRESH_RATE);
pub const SLEEP_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_CONFIG_NAME: &str = "deadlocked.toml";
pub const DEFAULT_URL: &str = "localhost:6346";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub aim: AimConfig,
    pub player: PlayerConfig,
    pub hud: HudConfig,
    pub radar: RadarConfig,
    pub misc: UnsafeConfig,
    pub accent_color: Color32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aim: AimConfig::default(),
            player: PlayerConfig::default(),
            hud: HudConfig::default(),
            radar: RadarConfig::default(),
            misc: UnsafeConfig::default(),
            accent_color: Colors::BLUE,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponConfig {
    pub aimbot: AimbotConfig,
    pub rcs: RcsConfig,
    pub triggerbot: TriggerbotConfig,
}

impl WeaponConfig {
    pub fn enabled(enabled: bool) -> Self {
        let aimbot = AimbotConfig {
            enable_override: enabled,
            ..Default::default()
        };
        Self {
            aimbot,
            rcs: RcsConfig::default(),
            triggerbot: TriggerbotConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub mode: KeyMode,
    pub target_friendlies: bool,
    pub distance_adjusted_fov: bool,
    pub start_bullet: i32,
    pub visibility_check: bool,
    pub smoke_wall_check: bool,
    pub flash_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub bones: Vec<Bones>,
    pub targeting_mode: TargetingMode,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            mode: KeyMode::Shoot,
            target_friendlies: false,
            distance_adjusted_fov: true,
            start_bullet: 1,
            visibility_check: true,
            smoke_wall_check: true,
            flash_check: true,
            fov: 1.8,
            smooth: 8.5,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
            targeting_mode: TargetingMode::Fov,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RcsConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub smooth: f32,
}

impl Default for RcsConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            smooth: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum KeyMode {
    Hold,
    Toggle,
    Shoot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TargetingMode {
    Fov,
    Distance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub delay: RangeInclusive<u64>,
    pub shot_duration: u64,
    pub mode: KeyMode,
    pub flash_check: bool,
    pub smoke_wall_check: bool,
    pub scope_check: bool,
    pub velocity_check: bool,
    pub velocity_threshold: f32,
    pub aim_assist: bool,
    pub aim_fov: f32,
    pub aim_smooth: f32,
    pub fire_fov: f32,
    pub head_only: bool,
}

impl Default for TriggerbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            delay: 85..=145,
            shot_duration: 16,
            mode: KeyMode::Shoot,
            flash_check: true,
            smoke_wall_check: true,
            scope_check: true,
            velocity_check: false,
            velocity_threshold: 220.0,
            aim_assist: true,
            aim_fov: 1.8,
            aim_smooth: 4.0,
            fire_fov: 0.22,
            head_only: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimConfig {
    pub aimbot_hotkey: KeyCode,
    pub triggerbot_hotkey: KeyCode,
    pub global: WeaponConfig,
    pub weapons: HashMap<Weapon, WeaponConfig>,
}

impl Default for AimConfig {
    fn default() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            aimbot_hotkey: KeyCode::MouseLeft,
            triggerbot_hotkey: KeyCode::MouseLeft,
            global: WeaponConfig::enabled(true),
            weapons,
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum DrawMode {
    None,
    Health,
    Color,
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum BoxMode {
    Gap,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub enabled: bool,
    pub esp_hotkey: KeyCode,
    pub show_friendlies: bool,
    pub draw_box: DrawMode,
    pub box_mode: BoxMode,
    pub box_visible_color: Color32,
    pub box_invisible_color: Color32,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color32,
    pub head_circle: bool,
    pub health_bar: bool,
    pub armor_bar: bool,
    pub player_name: bool,
    pub weapon_icon: bool,
    pub tags: bool,
    pub visible_only: bool,
    pub sound: SoundConfig,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            esp_hotkey: KeyCode::X,
            show_friendlies: false,
            draw_box: DrawMode::Color,
            box_mode: BoxMode::Gap,
            box_visible_color: Color32::WHITE,
            box_invisible_color: Color32::RED,
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color32::WHITE,
            head_circle: true,
            health_bar: true,
            armor_bar: true,
            player_name: true,
            weapon_icon: true,
            tags: true,
            visible_only: false,
            sound: SoundConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SoundConfig {
    pub enabled: bool,
    pub footstep_diameter: f32,
    pub gunshot_diameter: f32,
    pub weapon_diameter: f32,
    pub fadeout_start: f32,
    pub fadeout_duration: f32,
    pub show_visible: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            footstep_diameter: crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT,
            gunshot_diameter: crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT,
            weapon_diameter: crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT,
            fadeout_start: 1.0,
            fadeout_duration: 1.0,
            show_visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_timer: bool,
    pub fov_circle: bool,
    pub sniper_crosshair: bool,
    pub crosshair_color: Color32,
    pub dropped_weapons: bool,
    pub grenade_trails: bool,
    pub smoke_trail_color: Color32,
    pub molotov_trail_color: Color32,
    pub incendiary_trail_color: Color32,
    pub flash_trail_color: Color32,
    pub he_trail_color: Color32,
    pub decoy_trail_color: Color32,
    pub text_outline: bool,
    pub text_color: Color32,
    pub line_width: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub overlay_refresh_rate: u64,
    pub data_refresh_rate: u64,
    pub debug: bool,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_timer: true,
            fov_circle: false,
            sniper_crosshair: true,
            crosshair_color: Color32::WHITE,
            dropped_weapons: true,
            grenade_trails: true,
            smoke_trail_color: Color32::LIGHT_GRAY,
            molotov_trail_color: Color32::RED,
            incendiary_trail_color: Color32::ORANGE,
            flash_trail_color: Color32::WHITE,
            he_trail_color: Color32::DARK_GRAY,
            decoy_trail_color: Color32::PURPLE,
            text_outline: true,
            text_color: Colors::TEXT,
            line_width: 2.0,
            font_size: 16.0,
            icon_size: 20.0,
            overlay_refresh_rate: 120,
            data_refresh_rate: 90,
            debug: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RadarConfig {
    pub enabled: bool,
    pub url: String,
}

impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: DEFAULT_URL.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UnsafeConfig {
    pub no_flash: bool,
    pub max_flash_alpha: f32,
    pub fov_changer: bool,
    pub desired_fov: u32,
    pub silent_aim: bool,
    pub telemetry_enabled: bool,
    pub auto_counter_strafe: bool,
    pub counter_strafe_tap_ms: u64,
    pub counter_strafe_speed_threshold: f32,
    pub no_smoke: bool,
    pub change_smoke_color: bool,
    pub smoke_color: Color32,
}

impl Default for UnsafeConfig {
    fn default() -> Self {
        Self {
            no_flash: false,
            max_flash_alpha: 127.0,
            fov_changer: false,
            desired_fov: 90,
            silent_aim: false,
            telemetry_enabled: true,
            auto_counter_strafe: true,
            counter_strafe_tap_ms: 18,
            counter_strafe_speed_threshold: 22.0,
            no_smoke: false,
            change_smoke_color: false,
            smoke_color: Color32::RED,
        }
    }
}

pub static BASE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("deadlocked"))
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.join("configs");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub fn parse_config(path: &Path) -> Config {
    if !path.exists() || path.is_dir() {
        return Config::default();
    }

    let Ok(config_string) = read_to_string(path) else {
        return Config::default();
    };

    let config = toml::from_str(&config_string);
    if config.is_err() {
        log::warn!("config file invalid");
    } else if let Some(file_name) = path.file_name() {
        log::info!("loaded config {:?}", file_name);
    }
    config.unwrap_or_default()
}

pub fn write_config(config: &Config, path: &Path) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(path, out);
}

pub fn delete_config(path: &Path) {
    if !path.exists() {
        return;
    }

    if std::fs::remove_file(path).is_ok()
        && let Some(file_name) = path.file_name()
    {
        log::info!("deleted config {:?}", file_name);
    }
}

pub fn available_configs() -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(8);
    let Ok(dir) = std::fs::read_dir::<&Path>(CONFIG_PATH.as_ref()) else {
        return files;
    };

    for path in dir {
        let Ok(file) = path else {
            continue;
        };
        let Ok(file_type) = file.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !file_name.ends_with(".toml") {
            continue;
        }
        files.push(file.path())
    }
    if files.is_empty() {
        let path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        write_config(&Config::default(), &path);
        files.push(path);
    }
    files
}
