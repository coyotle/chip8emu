use rodio::Source;
use std::f32::consts::PI;

/// Generate Sine Wave
pub struct SineWave {
    freq: f32,
    sample_rate: u32,
    current_sample: u64,
}

impl SineWave {
    pub fn new(freq: f32) -> Self {
        Self {
            freq,
            sample_rate: 44100,
            current_sample: 0,
        }
    }
}

impl Iterator for SineWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let time = self.current_sample as f32 / self.sample_rate as f32;
        let value = (2.0 * PI * self.freq * time).sin();
        self.current_sample = self.current_sample.wrapping_add(1);
        Some(value)
    }
}

impl Source for SineWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
