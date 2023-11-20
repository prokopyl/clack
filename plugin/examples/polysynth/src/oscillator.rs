use std::f32::consts::PI;

#[derive(Copy, Clone)]
pub struct SquareOscillator {
    frequency_to_phase_increment_ratio: f32,
}

impl SquareOscillator {
    #[inline]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency_to_phase_increment_ratio: 2.0 * PI / sample_rate,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        todo!()
    }

    #[inline]
    pub fn set_note_number(&mut self, _new_note_number: u8) {
        todo!()
    }

    #[inline]
    pub fn set_frequency(&mut self, _new_frequency: f32) {
        todo!()
    }

    #[inline]
    pub fn add_next_samples_to_buffer(&mut self, buf: &mut [f32]) {
        todo!()
    }
}
