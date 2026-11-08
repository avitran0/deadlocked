use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use crate::{
    audio::Audio,
    config::{BASE_PATH, hud::QuakeSoundsConfig},
};

/// announcer sounds built into the binary, so the feature works out of the box
const BUILTIN_SOUNDS: [(QuakeSound, &[u8]); 12] = [
    (
        QuakeSound::FirstBlood,
        include_bytes!("../assets/audio/quake/firstblood.wav"),
    ),
    (
        QuakeSound::Headshot,
        include_bytes!("../assets/audio/quake/headshot.wav"),
    ),
    (
        QuakeSound::Humiliation,
        include_bytes!("../assets/audio/quake/humiliation.wav"),
    ),
    (
        QuakeSound::DoubleKill,
        include_bytes!("../assets/audio/quake/doublekill.wav"),
    ),
    (
        QuakeSound::TripleKill,
        include_bytes!("../assets/audio/quake/triplekill.wav"),
    ),
    (
        QuakeSound::MultiKill,
        include_bytes!("../assets/audio/quake/multikill.wav"),
    ),
    (
        QuakeSound::MonsterKill,
        include_bytes!("../assets/audio/quake/monsterkill.wav"),
    ),
    (
        QuakeSound::KillingSpree,
        include_bytes!("../assets/audio/quake/killingspree.wav"),
    ),
    (
        QuakeSound::Rampage,
        include_bytes!("../assets/audio/quake/rampage.wav"),
    ),
    (
        QuakeSound::Dominating,
        include_bytes!("../assets/audio/quake/dominating.wav"),
    ),
    (
        QuakeSound::Unstoppable,
        include_bytes!("../assets/audio/quake/unstoppable.wav"),
    ),
    (
        QuakeSound::Godlike,
        include_bytes!("../assets/audio/quake/godlike.wav"),
    ),
];

/// how often the sounds folder is checked for changes
const SCAN_INTERVAL: Duration = Duration::from_secs(2);

const SUPPORTED_EXTENSIONS: [&str; 4] = ["wav", "mp3", "ogg", "flac"];

/// kills without dying required for each spree sound, ascending
const SPREE_THRESHOLDS: [(u64, QuakeSound); 5] = [
    (3, QuakeSound::KillingSpree),
    (5, QuakeSound::Rampage),
    (7, QuakeSound::Dominating),
    (9, QuakeSound::Unstoppable),
    (11, QuakeSound::Godlike),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum QuakeSound {
    FirstBlood,
    Headshot,
    Humiliation,
    DoubleKill,
    TripleKill,
    MultiKill,
    MonsterKill,
    KillingSpree,
    Rampage,
    Dominating,
    Unstoppable,
    Godlike,
}

impl QuakeSound {
    pub const ALL: [QuakeSound; 12] = [
        QuakeSound::FirstBlood,
        QuakeSound::Headshot,
        QuakeSound::Humiliation,
        QuakeSound::DoubleKill,
        QuakeSound::TripleKill,
        QuakeSound::MultiKill,
        QuakeSound::MonsterKill,
        QuakeSound::KillingSpree,
        QuakeSound::Rampage,
        QuakeSound::Dominating,
        QuakeSound::Unstoppable,
        QuakeSound::Godlike,
    ];

    /// expected file stem inside the sounds folder
    pub fn file_stem(self) -> &'static str {
        match self {
            QuakeSound::FirstBlood => "firstblood",
            QuakeSound::Headshot => "headshot",
            QuakeSound::Humiliation => "humiliation",
            QuakeSound::DoubleKill => "doublekill",
            QuakeSound::TripleKill => "triplekill",
            QuakeSound::MultiKill => "multikill",
            QuakeSound::MonsterKill => "monsterkill",
            QuakeSound::KillingSpree => "killingspree",
            QuakeSound::Rampage => "rampage",
            QuakeSound::Dominating => "dominating",
            QuakeSound::Unstoppable => "unstoppable",
            QuakeSound::Godlike => "godlike",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            QuakeSound::FirstBlood => "First Blood",
            QuakeSound::Headshot => "Headshot",
            QuakeSound::Humiliation => "Humiliation (Knife)",
            QuakeSound::DoubleKill => "Double Kill",
            QuakeSound::TripleKill => "Triple Kill",
            QuakeSound::MultiKill => "Multi Kill",
            QuakeSound::MonsterKill => "Monster Kill",
            QuakeSound::KillingSpree => "Killing Spree (3)",
            QuakeSound::Rampage => "Rampage (5)",
            QuakeSound::Dominating => "Dominating (7)",
            QuakeSound::Unstoppable => "Unstoppable (9)",
            QuakeSound::Godlike => "Godlike (11)",
        }
    }
}

pub struct QuakeSounds {
    sounds: HashMap<QuakeSound, Vec<u8>>,
    /// sounds replaced by a file from the sounds folder
    overridden: HashSet<QuakeSound>,
    sounds_path: PathBuf,
    scanned: bool,
    last_scan: Option<Instant>,
    last_mtime: Option<SystemTime>,
    last_override_count: usize,

    // event tracking
    last_map: String,
    last_kill_sequence: u64,
    last_headshot_sequence: u64,
    last_knife_sequence: u64,
    last_round_sequence: u64,
    was_dead: bool,
    first_blood_done: bool,
    /// kill count of the running multi kill chain and when it started
    multi_chain: Option<(u64, Instant)>,
    spree: u64,
}

impl QuakeSounds {
    pub fn new() -> Self {
        let sounds_path = BASE_PATH.join("sounds");
        if !sounds_path.exists() {
            let _ = fs::create_dir_all(&sounds_path);
        }

        let mut state = Self {
            sounds: HashMap::new(),
            overridden: HashSet::new(),
            sounds_path,
            scanned: false,
            last_scan: None,
            last_mtime: None,
            last_override_count: 0,
            last_map: String::new(),
            last_kill_sequence: 0,
            last_headshot_sequence: 0,
            last_knife_sequence: 0,
            last_round_sequence: 0,
            was_dead: false,
            first_blood_done: false,
            multi_chain: None,
            spree: 0,
        };
        state.reload();
        state.write_template();
        state
    }

    pub fn sounds_path(&self) -> &Path {
        &self.sounds_path
    }

    /// how many of the built in sounds are replaced by files from the folder
    pub fn folder_overrides(&self) -> usize {
        self.overridden.len()
    }

    /// called once per frame with the latest game data
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        audio: &Audio,
        config: &QuakeSoundsConfig,
        in_game: bool,
        map_name: &str,
        kill_sequence: u64,
        headshot_sequence: u64,
        knife_sequence: u64,
        round_sequence: u64,
        local_health: i32,
    ) {
        self.refresh_sounds();

        if !config.enabled || !in_game {
            // keep the counters in sync without triggering anything,
            // a spree is void once the round or life it belonged to is over
            self.last_kill_sequence = kill_sequence;
            self.last_headshot_sequence = headshot_sequence;
            self.last_knife_sequence = knife_sequence;
            self.last_round_sequence = round_sequence;
            self.multi_chain = None;
            self.spree = 0;
            self.was_dead = local_health <= 0;
            audio.clear_queue();
            return;
        }

        // new map means a new session
        if !self.last_map.is_empty() && map_name != self.last_map {
            self.first_blood_done = false;
            self.multi_chain = None;
            self.spree = 0;
            self.last_round_sequence = round_sequence;
        }
        self.last_map = map_name.to_string();

        // a new round makes first blood available again
        if round_sequence != self.last_round_sequence {
            self.last_round_sequence = round_sequence;
            self.first_blood_done = false;
            self.multi_chain = None;
        }

        // dying breaks sprees and multi kill chains
        let dead = local_health <= 0;
        if dead && !self.was_dead {
            self.spree = 0;
            self.multi_chain = None;
        }
        self.was_dead = dead;

        let kill_count = kill_sequence.saturating_sub(self.last_kill_sequence);
        let headshot_count = headshot_sequence.saturating_sub(self.last_headshot_sequence);
        let knife_count = knife_sequence.saturating_sub(self.last_knife_sequence);
        self.last_kill_sequence = kill_sequence;
        self.last_headshot_sequence = headshot_sequence;
        self.last_knife_sequence = knife_sequence;

        if kill_count == 0 {
            return;
        }

        let now = Instant::now();
        let mut events: Vec<QuakeSound> = Vec::with_capacity(3);

        if !self.first_blood_done {
            self.first_blood_done = true;
            events.push(QuakeSound::FirstBlood);
        }

        // kills within the configured window chain up into multi kills
        let chain = match self.multi_chain {
            Some((count, started))
                if now.duration_since(started).as_secs_f32()
                    <= config.multi_kill_window.max(0.1) =>
            {
                count + kill_count
            }
            _ => kill_count,
        };
        self.multi_chain = Some((chain, now));

        let multi_kill = match chain {
            2 => Some(QuakeSound::DoubleKill),
            3 => Some(QuakeSound::TripleKill),
            4 => Some(QuakeSound::MultiKill),
            chain if chain >= 5 => Some(QuakeSound::MonsterKill),
            _ => None,
        };
        if let Some(sound) = multi_kill {
            events.push(sound);
        }

        let previous_spree = self.spree;
        self.spree = self.spree.saturating_add(kill_count);
        for (threshold, sound) in SPREE_THRESHOLDS {
            if previous_spree < threshold && self.spree >= threshold {
                events.push(sound);
            }
        }

        // queued last so announcements play after the kill itself
        if knife_count > 0 {
            events.push(QuakeSound::Humiliation);
        }
        if headshot_count > 0 {
            events.push(QuakeSound::Headshot);
        }

        for event in events {
            if let Some(bytes) = self.sounds.get(&event) {
                audio.queue(bytes, config.volume);
            }
        }
    }

    /// reload the sounds folder if it changed since the last scan
    fn refresh_sounds(&mut self) {
        let now = Instant::now();
        if self
            .last_scan
            .is_some_and(|last| now.duration_since(last) < SCAN_INTERVAL)
        {
            return;
        }
        self.last_scan = Some(now);

        let mtime = fs::metadata(&self.sounds_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        if self.scanned && mtime == self.last_mtime {
            return;
        }
        self.reload();
    }

    /// rebuild the sound map: built in sounds first, folder files win
    fn reload(&mut self) {
        let mut sounds: HashMap<QuakeSound, Vec<u8>> = BUILTIN_SOUNDS
            .iter()
            .map(|(sound, bytes)| (*sound, bytes.to_vec()))
            .collect();
        let overrides = scan_sounds(&self.sounds_path);
        let overridden: HashSet<QuakeSound> = overrides.keys().copied().collect();
        for (sound, bytes) in overrides {
            sounds.insert(sound, bytes);
        }

        if overridden.len() != self.last_override_count {
            utils::info!(
                "quake sounds: {}/{} built in, {} overridden by {}",
                QuakeSound::ALL.len() - overridden.len(),
                QuakeSound::ALL.len(),
                overridden.len(),
                self.sounds_path.display()
            );
        }

        self.sounds = sounds;
        self.overridden = overridden;
        self.last_override_count = self.overridden.len();
        self.scanned = true;
        self.last_mtime = fs::metadata(&self.sounds_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
    }

    /// create a small readme inside the sounds folder explaining the overrides
    fn write_template(&self) {
        let path = self.sounds_path.join("README.txt");
        if path.exists() {
            return;
        }

        let mut content = String::from(
            "deadlocked quake sounds\n\
             =======================\n\
             All announcer sounds are built into the binary, so nothing is\n\
             required here. Drop your own sounds into this folder to replace\n\
             individual built in ones, everything else stays as it is.\n\n\
             Supported formats: wav, mp3, ogg, flac\n\
             Changes to this folder are picked up automatically.\n\n\
             File name           Event\n",
        );
        for sound in QuakeSound::ALL {
            content.push_str(&format!(
                "{:<20}{}\n",
                format!("{}.wav", sound.file_stem()),
                sound.label()
            ));
        }

        let _ = fs::write(path, content);
    }
}

fn scan_sounds(dir: &Path) -> HashMap<QuakeSound, Vec<u8>> {
    let mut sounds = HashMap::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return sounds;
    };

    let mut files: HashMap<String, PathBuf> = HashMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        files.insert(stem.to_lowercase(), path);
    }

    for sound in QuakeSound::ALL {
        for extension in SUPPORTED_EXTENSIONS {
            let Some(path) = files.get(&format!("{}.{}", sound.file_stem(), extension)) else {
                continue;
            };
            match fs::read(path) {
                Ok(bytes) => {
                    if let Err(error) = Audio::validate(bytes.clone()) {
                        utils::warn!("quake sound {:?} is not decodable: {error}", path);
                    } else {
                        sounds.insert(sound, bytes);
                    }
                    break;
                }
                Err(error) => {
                    utils::warn!("failed to read quake sound {:?}: {error}", path);
                }
            }
        }
    }

    sounds
}
