use std::io::Cursor;

use rodio::{buffer::SamplesBuffer, Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

use crate::config::player::SoundConfig;

const HIT_SOUND: &[u8] = include_bytes!("../../assets/hitSound.wav");
const KILL_SOUND: &[u8] = include_bytes!("../../assets/killSound.wav");

pub struct AudioPlayer {
    stream: OutputStream,
    hit_volume: f32,
    kill_volume: f32,
    hit_sound: Option<SamplesBuffer>,
    kill_sound: Option<SamplesBuffer>,
    hit_path: String,
    kill_path: String,
}

impl AudioPlayer {
    pub fn new(config: &SoundConfig) -> Option<Self> {
        let hit_sound = load_sound(&config.hit_path, HIT_SOUND);
        let kill_sound = load_sound(&config.kill_path, KILL_SOUND);

        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|error| utils::error!("failed to initialize audio output: {error}"))
            .ok()?;

        Some(Self {
            stream,
            hit_volume: config.hit_volume,
            kill_volume: config.kill_volume,
            hit_sound,
            kill_sound,
            hit_path: config.hit_path.clone(),
            kill_path: config.kill_path.clone(),
        })
    }

    pub fn set_hit_volume(&mut self, volume: f32) {
        self.hit_volume = volume;
    }

    pub fn set_kill_volume(&mut self, volume: f32) {
        self.kill_volume = volume;
    }

    pub fn set_hit_sound(&mut self, path: &str) {
        if self.hit_path != path {
            self.hit_sound = load_sound(path, HIT_SOUND);
            self.hit_path = path.to_owned();
        }
    }

    pub fn set_kill_sound(&mut self, path: &str) {
        if self.kill_path != path {
            self.kill_sound = load_sound(path, KILL_SOUND);
            self.kill_path = path.to_owned();
        }
    }

    pub fn update(&mut self, config: &SoundConfig) {
        self.set_hit_volume(config.hit_volume);
        self.set_kill_volume(config.kill_volume);
        self.set_hit_sound(&config.hit_path);
        self.set_kill_sound(&config.kill_path);
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
        let sink = Sink::connect_new(self.stream.mixer());
        sink.set_volume(volume);
        sink.append(sound.clone());
        sink.detach();
    }
}

fn load_sound(path: &str, default_bytes: &'static [u8]) -> Option<SamplesBuffer> {
    let trimmed = path.trim();
    if !trimmed.is_empty()
        && let Ok(bytes) = std::fs::read(trimmed)
        && let Some(sound) = decode_sound(&bytes)
    {
        return Some(sound);
    }
    decode_sound(default_bytes)
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
