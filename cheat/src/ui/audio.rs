use std::io::Cursor;

use rodio::{buffer::SamplesBuffer, Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

const HIT_SOUND: &[u8] = include_bytes!("../../assets/hitSound.mp3");
const KILL_SOUND: &[u8] = include_bytes!("../../assets/killSound.mp3");

pub struct AudioPlayer {
    _stream: OutputStream,
    hit_sink: Sink,
    kill_sink: Sink,
    hit_sound: SamplesBuffer,
    kill_sound: SamplesBuffer,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        let hit_sound = decode_sound(HIT_SOUND)?;
        let kill_sound = decode_sound(KILL_SOUND)?;

        OutputStreamBuilder::open_default_stream()
            .map(|stream| {
                let hit_sink = Sink::connect_new(stream.mixer());
                let kill_sink = Sink::connect_new(stream.mixer());
                Self {
                    _stream: stream,
                    hit_sink,
                    kill_sink,
                    hit_sound,
                    kill_sound,
                }
            })
            .map_err(|error| utils::error!("failed to initialize audio output: {error}"))
            .ok()
    }

    pub fn set_hit_volume(&self, volume: f32) {
        self.hit_sink.set_volume(volume);
    }

    pub fn set_kill_volume(&self, volume: f32) {
        self.kill_sink.set_volume(volume);
    }

    pub fn play_hit(&self) {
        self.play(&self.hit_sink, &self.hit_sound);
    }

    pub fn play_kill(&self) {
        self.play(&self.kill_sink, &self.kill_sound);
    }

    fn play(&self, sink: &Sink, sound: &SamplesBuffer) {
        // Keep only the latest feedback sound: queued stale sounds feel delayed.
        if !sink.empty() {
            sink.clear();
        }
        sink.append(sound.clone());
    }
}

fn decode_sound(bytes: &'static [u8]) -> Option<SamplesBuffer> {
    let decoder = Decoder::try_from(Cursor::new(bytes))
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
