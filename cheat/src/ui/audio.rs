use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};

const HIT_SOUND: &[u8] = include_bytes!("../../assets/hitSound.mp3");
const KILL_SOUND: &[u8] = include_bytes!("../../assets/killSound.mp3");

pub struct AudioPlayer {
    stream: OutputStream,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        OutputStreamBuilder::open_default_stream()
            .map(|stream| Self { stream })
            .map_err(|error| utils::error!("failed to initialize audio output: {error}"))
            .ok()
    }

    pub fn play_hit(&self) {
        self.play(HIT_SOUND);
    }

    pub fn play_kill(&self) {
        self.play(KILL_SOUND);
    }

    fn play(&self, bytes: &'static [u8]) {
        let Ok(source) = Decoder::try_from(Cursor::new(bytes)) else {
            utils::error!("failed to decode sound file");
            return;
        };
        let sink = Sink::connect_new(self.stream.mixer());
        sink.append(source);
        sink.detach();
    }
}