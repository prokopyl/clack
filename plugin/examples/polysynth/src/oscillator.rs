use std::f32::consts::{PI, TAU};

/// A very basic Square Wave oscillator.
///
/// This whole implementation is from https://www.martin-finke.de/articles/audio-plugins-008-synthesizing-waveforms/.
#[derive(Copy, Clone)]
pub struct SquareOscillator {
    frequency_to_phase_increment_ratio: f32,
    phase_increment: f32,
    current_phase: f32,
}

impl SquareOscillator {
    #[inline]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency_to_phase_increment_ratio: 2.0 * PI / sample_rate,
            phase_increment: 1.0,
            current_phase: 0.,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.current_phase = 0.;
    }

    #[inline]
    pub fn set_note_number(&mut self, new_note_number: u8) {
        self.set_frequency(440.0 * 2.0f32.powf((new_note_number as f32 - 69.0) / 12.0));
    }

    #[inline]
    pub fn set_frequency(&mut self, new_frequency: f32) {
        self.phase_increment = new_frequency * self.frequency_to_phase_increment_ratio;
    }

    #[inline]
    pub fn add_next_samples_to_buffer(&mut self, buf: &mut [f32]) {
        // Keep enough headroom to play a few notes at once without "clipping".
        const VOLUME: f32 = 0.2;

        for value in buf {
            if self.current_phase <= PI {
                *value += VOLUME;
            } else {
                *value -= VOLUME;
            }

            self.current_phase += self.phase_increment;
            while self.current_phase > TAU {
                self.current_phase -= TAU;
            }
        }
    }
}
