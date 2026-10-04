use std::io::Cursor;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Source};

use crate::config::hud::HitmarkerSound;

pub struct Audio {
    sink: MixerDeviceSink,
}

impl Audio {
    pub fn new() -> Self {
        let mut sink = DeviceSinkBuilder::open_default_sink().unwrap_or_else(|error| {
            utils::error!("failed to initialize audio output: {error}");
            std::process::exit(1);
        });
        sink.log_on_drop(false);
        Self { sink }
    }

    pub fn play(&self, bytes: &[u8], volume: f32) {
        let Ok(source) = Decoder::try_from(Cursor::new(bytes.to_vec())) else {
            utils::warn!("failed to decode hitmarker sound");
            return;
        };
        self.sink
            .mixer()
            .add(source.amplify(volume.clamp(0.0, 10.0)));
    }

    pub fn validate(bytes: Vec<u8>) -> Result<(), String> {
        Decoder::try_from(Cursor::new(bytes))
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

pub fn builtin_sound(sound: HitmarkerSound) -> &'static [u8] {
    match sound {
        HitmarkerSound::Beep => include_bytes!("../assets/audio/beep.wav"),
        HitmarkerSound::Bell => include_bytes!("../assets/audio/bell.wav"),
        HitmarkerSound::BulletCasing => include_bytes!("../assets/audio/bullet_casing.wav"),
        HitmarkerSound::Click => include_bytes!("../assets/audio/click.wav"),
        HitmarkerSound::Clink => include_bytes!("../assets/audio/clink.mp3"),
        HitmarkerSound::KnifeImpact => include_bytes!("../assets/audio/knife_impact.wav"),
        HitmarkerSound::RubberTire => include_bytes!("../assets/audio/rubber_tire.wav"),
        HitmarkerSound::WaterDrip => include_bytes!("../assets/audio/water_drip.wav"),
        HitmarkerSound::Wood => include_bytes!("../assets/audio/wood.mp3"),
        HitmarkerSound::Custom => include_bytes!("../assets/audio/water_drip.wav"),
    }
}
