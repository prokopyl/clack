use crate::oscillator::SquareOscillator;
use clack_plugin::prelude::UnknownEvent;

#[derive(Copy, Clone)]
struct Voice {
    oscillator: SquareOscillator,
}

pub struct PolyOscillator {
    voice_buffer: Box<[Voice]>,
    active_voice_count: usize,
}

impl PolyOscillator {
    pub fn new(voice_count: usize, sample_rate: f32) -> Self {
        Self {
            voice_buffer: vec![
                Voice {
                    oscillator: SquareOscillator::new(sample_rate)
                };
                voice_count
            ]
            .into_boxed_slice(),
            active_voice_count: 0,
        }
    }

    pub fn process_event(&mut self, event: &UnknownEvent) {
        todo!()
    }

    pub fn generate_next_samples(&mut self, output_buffer: &mut [f32]) {
        todo!()
    }
}
