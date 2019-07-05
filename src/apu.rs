use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::thread;

use cpal;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::distributions::Uniform;

static ATTACK: [u32; 16] = [2, 8, 16, 24, 38, 56, 68, 80, 100, 250, 500, 800, 1000, 3000, 5000, 8000];
static DECAY: [u32; 16] = [6, 24, 48, 72, 114, 168, 204, 240, 300, 750, 1500, 2400, 3000, 9000, 15000, 24000];
static RELEASE: [u32; 16] = [6, 24, 48, 72, 114, 168, 204, 240, 300, 750, 1500, 2400, 3000, 9000, 15000, 24000];

pub struct Apu {
    max_volume: f32,

    _handle: thread::JoinHandle<()>,
    event_loop: Arc<cpal::EventLoop>,
    stream_id: cpal::StreamId,
    gen: Arc<Mutex<Generator>>,

    sample_rate: u32,

    wave: Wave,
    volume: f32,
    sustain: f32,
    attack: usize,
    decay: usize,
    release: usize,
}

impl Apu {
    /// Create a new audio processing
    pub fn new(max_volume: f32) -> Apu {
        let device = cpal::default_output_device().expect("Failed to get default output device");
        let format = device.default_output_format().expect("Failed to get default output format");

        let event_loop = Arc::new(cpal::EventLoop::new());
        let stream_id = event_loop.build_output_stream(&device, &format).expect("Failed to build output stream");

        let gen = Arc::new(Mutex::new(Generator::silence()));

        let sample_rate = format.sample_rate.0;

        let handle = {
            let event_loop = event_loop.clone();
            let gen = gen.clone();
            thread::spawn(move || event_loop.run(|stream_id, data| {
                let ref mut gen = *gen.lock().unwrap();

                if gen.is_finished() {
                    event_loop.pause_stream(stream_id);
                }

                match data {
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                        for (sample, value) in buffer.chunks_mut(format.channels as usize).zip(gen) {
                            let value = ((value * 0.5 + 0.5) * u16::max_value() as f32) as u16;
                            for out in sample.iter_mut() { *out = value }
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                        for (sample, value) in buffer.chunks_mut(format.channels as usize).zip(gen) {
                            let value = (value * i16::max_value() as f32) as i16;
                            for out in sample.iter_mut() { *out = value }
                        }
                    },
                    cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                        for (sample, value) in buffer.chunks_mut(format.channels as usize).zip(gen) {
                            for out in sample.iter_mut() { *out = value }
                        }
                    },
                    _ => {},
                }
            }))
        };

        Apu {
            max_volume,

            _handle: handle,
            event_loop,
            stream_id,
            gen,

            sample_rate,

            wave: Wave::Pulse,
            volume: 1.0,
            sustain: 1.0,
            attack: 0,
            decay: 0,
            release: 0,
        }
    }

    pub fn settings(&mut self, attack: u8, decay: u8, sustain: u8, release: u8, volume: u8, wave: u8) {
        self.attack = attack as usize;
        self.decay = decay as usize;
        self.release = release as usize;
        self.sustain = sustain as f32 / 15.0;
        self.volume = volume as f32 / 15.0;
        self.wave = Wave::from_byte(wave).unwrap_or(Wave::Pulse);
    }

    /// Play a sound with a frequency given in hz for a duration given in ms
    pub fn play(&mut self, frequency: u16, duration: u16, adsr: bool) {
        let volume = self.volume;
        let samples_duration = duration as u32 * self.sample_rate / 1000;

        let (sustain, samples_attack, samples_decay, samples_release, samples_total, wave);
        if adsr {
            sustain = self.sustain;
            samples_attack = ATTACK[self.attack] * self.sample_rate / 1000;
            samples_decay = DECAY[self.decay] * self.sample_rate / 1000;
            samples_release = RELEASE[self.release] * self.sample_rate / 1000;
            samples_total = samples_attack + samples_decay + max(0, samples_duration as i64 - samples_attack as i64 - samples_decay as i64) as u32 + samples_release;
            wave = self.wave.clone();
        } else {
            sustain = volume;
            samples_attack = 0;
            samples_decay = 0;
            samples_release = 0;
            samples_total = samples_duration;
            wave = Wave::Pulse;
        }

        let mut gen = self.gen.lock().unwrap();
        *gen = Generator {
            volume: volume * self.max_volume,
            sustain: sustain * self.max_volume,
            wave: wave,

            samples_attack: samples_attack as f32,
            samples_decay: samples_decay as f32,
            samples_release: samples_release as f32,

            samples_total: samples_total as f32,
            samples_period: (self.sample_rate / frequency as u32) as f32,

            samples_count: 0.0,
        };

        self.event_loop.play_stream(self.stream_id.clone());
    }

    /// Stop the currently playing sound
    pub fn stop(&mut self) {
        self.event_loop.pause_stream(self.stream_id.clone());
    }
}

#[derive(Clone)]
enum Wave {
    Triangle,
    Sawtooth,
    Pulse,
    Noise(SmallRng),
}

impl Wave {
    fn from_byte(byte: u8) -> Result<Wave, String> {
        Ok(match byte {
            0 => Wave::Triangle,
            1 => Wave::Sawtooth,
            2 => Wave::Pulse,
            3 => Wave::Noise(SmallRng::from_entropy()),
            _ => return Err(format!("Unknown Wave 0x{:02X}", byte)),
        })
    }

    fn sample(&mut self, index: f32, period: f32) -> f32 {
        match *self {
            Wave::Triangle => (4.0 / period) * ((index % period) - (period / 2.0)).abs() - 1.0,
            Wave::Sawtooth => (2.0 / period) * (index % period) - 1.0,
            Wave::Pulse => if index % period < (period / 2.0) { 1.0 } else { -1.0 },
            Wave::Noise(ref mut rng) => rng.sample(Uniform::new_inclusive(-1.0, 1.0)),
        }
    }
}

pub struct Generator {
    volume: f32,
    sustain: f32,
    wave: Wave,

    samples_attack: f32,
    samples_decay: f32,
    samples_release: f32,
    samples_total: f32,
    samples_period: f32,

    samples_count: f32,
}

impl Generator {
    fn silence() -> Generator {
        Generator {
            volume: 0.0,
            sustain: 0.0,
            wave: Wave::Pulse,
            samples_attack: 0.0,
            samples_decay: 0.0,
            samples_release: 0.0,
            samples_total: 0.0,
            samples_period: 0.0,
            samples_count: 0.0,
        }
    }

    fn is_finished(&self) -> bool {
        self.samples_count == self.samples_total
    }
}

impl Iterator for Generator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.is_finished() {
            return Some(0.0);
        }

        let mut sample = self.wave.sample(self.samples_count, self.samples_period);

        if self.samples_count < self.samples_attack { // Attack
            sample *= self.volume * (self.samples_count / self.samples_attack);
        } else if self.samples_count < self.samples_attack + self.samples_decay { // Decay
            sample *= self.sustain + (self.volume - self.sustain) * (1.0 - ((self.samples_count - self.samples_attack) / self.samples_decay));
        } else if self.samples_count >= self.samples_total - self.samples_release { // Release
            sample *= self.sustain * (1.0 - ((self.samples_count - (self.samples_total - self.samples_release)) / self.samples_release));
        } else {
            sample *= self.sustain;
        }

        self.samples_count += 1.0;
        Some(sample)
    }
}
