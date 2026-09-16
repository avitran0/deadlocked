use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source, buffer::SamplesBuffer};

const HIT_SOUND: &[u8] = include_bytes!("../../assets/hitSound.wav");
const KILL_SOUND: &[u8] = include_bytes!("../../assets/killSound.wav");

pub struct AudioPlayer {
    stream: OutputStream,
    hit_sound: SamplesBuffer,
    kill_sound: SamplesBuffer,
    hit_volume: f32,
    kill_volume: f32,
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        let hit_sound = decode_sound(HIT_SOUND)?;
        let kill_sound = decode_sound(KILL_SOUND)?;

        OutputStreamBuilder::open_default_stream()
            .map(|stream| Self {
                stream,
                hit_sound,
                kill_sound,
                hit_volume: 1.0,
                kill_volume: 1.0,
            })
            .map_err(|error| utils::error!("failed to initialize audio output: {error}"))
            .ok()
    }

    pub fn set_hit_volume(&mut self, volume: f32) {
        self.hit_volume = volume;
    }

    pub fn set_kill_volume(&mut self, volume: f32) {
        self.kill_volume = volume;
    }

    pub fn play_hit(&self) {
        self.play(&self.hit_sound, self.hit_volume);
    }

    pub fn play_kill(&self) {
        self.play(&self.kill_sound, self.kill_volume);
    }

    fn play(&self, sound: &SamplesBuffer, volume: f32) {
        // A new sink lets rapid hits overlap instead of cancelling the previous
        // feedback before the audio thread has had a chance to play it.
        let sink = Sink::connect_new(self.stream.mixer());
        sink.set_volume(volume);
        sink.append(sound.clone());
        sink.detach();
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
