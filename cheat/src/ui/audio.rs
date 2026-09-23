use std::{
    io::Cursor,
    path::Path,
    sync::{Arc, Mutex},
};

use include_dir::{include_dir, Dir};
use rodio::{
    buffer::SamplesBuffer, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source,
};

use crate::config::player::HitSoundConfig;

static AUDIO_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/assets/audio");

pub fn audio_options() -> impl Iterator<Item = &'static str> {
    AUDIO_DIR.files().filter_map(|file| {
        file.path()
            .file_name()
            .and_then(|name| name.to_str())
    })
}

pub fn first_audio() -> Option<&'static str> {
    audio_options().next()
}

pub fn normalize_audio_path(path: &str) -> String {
    if audio_options().any(|option| option == path) {
        path.to_owned()
    } else {
        first_audio().unwrap_or_default().to_owned()
    }
}

pub fn audio_display_name(path: &str) -> &str {
    Path::new(path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
}

pub struct AudioPlayer {
    stream: Arc<Mutex<Option<MixerDeviceSink>>>,
    stream_initializing: Arc<Mutex<bool>>,
    hit_volume: f32,
    kill_volume: f32,
    hit_sound: Option<SamplesBuffer>,
    kill_sound: Option<SamplesBuffer>,
    hit_path: String,
    kill_path: String,
}

impl AudioPlayer {
    pub fn new(hit_config: &HitSoundConfig, kill_config: &HitSoundConfig) -> Self {
        let hit_path = normalize_audio_path(&hit_config.path);
        let kill_path = normalize_audio_path(&kill_config.path);
        let hit_sound = load_sound(&hit_path);
        let kill_sound = load_sound(&kill_path);

        let player = Self {
            stream: Arc::new(Mutex::new(None)),
            stream_initializing: Arc::new(Mutex::new(false)),
            hit_volume: hit_config.volume,
            kill_volume: kill_config.volume,
            hit_sound,
            kill_sound,
            hit_path,
            kill_path,
        };
        player.ensure_stream(hit_config.enabled || kill_config.enabled);
        player
    }

    pub fn set_hit_volume(&mut self, volume: f32) {
        self.hit_volume = volume;
    }

    pub fn set_kill_volume(&mut self, volume: f32) {
        self.kill_volume = volume;
    }

    pub fn set_hit_sound(&mut self, path: &str) {
        let path = normalize_audio_path(path);
        if self.hit_path != path {
            self.hit_sound = load_sound(&path);
            self.hit_path = path;
        }
    }

    pub fn set_kill_sound(&mut self, path: &str) {
        let path = normalize_audio_path(path);
        if self.kill_path != path {
            self.kill_sound = load_sound(&path);
            self.kill_path = path;
        }
    }

    pub fn update(&mut self, hit_config: &HitSoundConfig, kill_config: &HitSoundConfig) {
        self.ensure_stream(hit_config.enabled || kill_config.enabled);
        self.set_hit_volume(hit_config.volume);
        self.set_kill_volume(kill_config.volume);
        self.set_hit_sound(&hit_config.path);
        self.set_kill_sound(&kill_config.path);
    }

    pub fn play_hit(&self) {
        if let Some(sound) = &self.hit_sound {
            self.play(sound, self.hit_volume);
        }
    }

    pub fn play_kill(&self) {
        if let Some(sound) = &self.kill_sound {
            self.play(sound, self.kill_volume);
        }
    }

    fn play(&self, sound: &SamplesBuffer, volume: f32) {
        let Ok(stream) = self.stream.lock() else {
            return;
        };
        let Some(stream) = stream.as_ref() else {
            return;
        };
        let player = Player::connect_new(stream.mixer());
        player.set_volume(volume);
        player.append(sound.clone());
        player.detach();
    }

    fn ensure_stream(&self, enabled: bool) {
        if !enabled {
            return;
        }

        let Ok(mut initializing) = self.stream_initializing.lock() else {
            return;
        };
        if *initializing {
            return;
        }
        if self.stream.lock().map(|stream| stream.is_some()).unwrap_or(true) {
            return;
        }
        *initializing = true;

        let stream = Arc::clone(&self.stream);
        let stream_initializing = Arc::clone(&self.stream_initializing);
        std::thread::spawn(move || {
            let result = DeviceSinkBuilder::from_default_device()
                .and_then(|builder| builder.open_sink_or_fallback())
                .map_err(|error| utils::error!("failed to initialize audio output: {error}"))
                .ok();
            if let Ok(mut stream) = stream.lock() {
                *stream = result;
            }
            if let Ok(mut initializing) = stream_initializing.lock() {
                *initializing = false;
            }
        });
    }
}

fn load_sound(path: &str) -> Option<SamplesBuffer> {
    let bytes = AUDIO_DIR
        .files()
        .find(|file| {
            file.path()
                .file_name()
                .and_then(|name| name.to_str())
                == Some(path)
        })
        .or_else(|| AUDIO_DIR.files().next())
        .map(|file| file.contents())?;
    decode_sound(bytes)
}

fn decode_sound(bytes: &[u8]) -> Option<SamplesBuffer> {
    let decoder = Decoder::try_from(Cursor::new(bytes.to_vec()))
        .map_err(|error| utils::error!("failed to decode sound file: {error}"))
        .ok()?;
    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    Some(SamplesBuffer::new(
        channels,
        sample_rate,
        decoder.collect::<Vec<_>>(),
    ))
}
