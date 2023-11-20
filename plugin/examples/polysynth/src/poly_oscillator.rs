use crate::oscillator::SquareOscillator;
use clack_plugin::events::event_types::*;
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::prelude::*;

#[derive(Copy, Clone)]
struct Voice {
    oscillator: SquareOscillator,
    note_number: u8,
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
                    oscillator: SquareOscillator::new(sample_rate),
                    note_number: 0
                };
                voice_count
            ]
            .into_boxed_slice(),
            active_voice_count: 0,
        }
    }

    fn start_new_voice(&mut self, new_note_key: u8) {
        // Skip the event if we are out of voices
        let Some(available_voice) = self.voice_buffer.get_mut(self.active_voice_count) else {
            return;
        };

        available_voice.oscillator.reset();
        available_voice.oscillator.set_note_number(new_note_key);
        available_voice.note_number = new_note_key;

        self.active_voice_count += 1;
    }

    fn stop_voice(&mut self, note_key: u8) {
        let voice_index = self
            .voice_buffer
            .iter()
            .position(|v| v.note_number == note_key);

        // Find the voice that is playing that note.
        if let Some(voice_index) = voice_index {
            // Swap the targeted voice with the last one.
            self.voice_buffer
                .swap(voice_index, self.active_voice_count - 1);

            // Remove the last voice from the active pool.
            self.active_voice_count -= 1;
        }
    }

    pub fn stop_all(&mut self) {
        self.active_voice_count = 0;
    }

    pub fn handle_event(&mut self, event: &UnknownEvent) {
        match event.as_core_event() {
            Some(CoreEventSpace::NoteOn(NoteOnEvent(note_event))) => {
                // Ignore invalid or negative note keys.
                let Ok(note_key) = u8::try_from(note_event.key()) else {
                    return;
                };

                self.start_new_voice(note_key);
            }
            Some(CoreEventSpace::NoteOff(NoteOffEvent(note_event))) => {
                // A -1 key means shutting off all notes.
                if note_event.key() == -1 {
                    self.stop_all();
                    return;
                }

                // Ignore invalid note keys.
                let Ok(note_key) = u8::try_from(note_event.key()) else {
                    return;
                };

                self.stop_voice(note_key)
            }
            _ => {}
        }
    }

    pub fn generate_next_samples(&mut self, output_buffer: &mut [f32], volume: f32) {
        for voice in &mut self.voice_buffer[..self.active_voice_count] {
            voice
                .oscillator
                .add_next_samples_to_buffer(output_buffer, volume);
        }
    }

    #[inline]
    pub fn has_active_voices(&self) -> bool {
        self.active_voice_count > 0
    }
}
