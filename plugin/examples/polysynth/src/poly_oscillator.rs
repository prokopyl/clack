//! Implementations and helpers for our polyphonic oscillator.

use crate::oscillator::SquareOscillator;
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::events::Match;
use clack_plugin::prelude::*;

/// A voice in the polyphonic oscillator.
#[derive(Copy, Clone)]
struct Voice {
    /// The oscillator itself.
    oscillator: SquareOscillator,
    /// The MIDI number of the note this voice is playing.
    /// This will be used to find this voice when a note end event is received.
    note_number: u8,
}

/// A simple polyphonic oscillator.
///
/// It tracks multiple oscillator voices, up to a given maximum.
///
/// This struct manages the buffer so that active voices are at the beginning, and inactive ones
/// at the end of the buffer. Then, to iterate only on the inactive voices, once can simply iterate
/// on the `0..active_voice_count` range.
pub struct PolyOscillator {
    /// The fixed buffer of voices.
    voice_buffer: Box<[Voice]>,
    /// The number of current
    active_voice_count: usize,
}

impl PolyOscillator {
    /// Initializes the oscillators with the given sample rate, and allocates the buffer to handle
    /// the given number of voices.
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

    /// Starts a new voice, playing the given MIDI note key.
    ///
    /// If there are no more voices available, this does nothing.
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

    /// Stops the first voice that is currently playing the given MIDI note key.
    ///
    /// If no matching voice is found, this does nothing.
    fn stop_voice(&mut self, note_key: u8) {
        let voice_index = self
            .voice_buffer
            .iter()
            .position(|v| v.note_number == note_key);

        // Find the voice that is playing that note.
        if let Some(voice_index) = voice_index {
            if voice_index >= self.active_voice_count {
                // the voice is not playing
                return;
            }

            // Swap the targeted voice with the last one.
            self.voice_buffer
                .swap(voice_index, self.active_voice_count - 1);

            // Remove the last voice from the active pool.
            self.active_voice_count -= 1;
        }
    }

    /// Stops all active voices.
    pub fn stop_all(&mut self) {
        self.active_voice_count = 0;
    }

    /// Handles the given input event.
    ///
    /// If the event is a note on event, then it will start a new voice for that note.
    /// If the event is a note off event, then it will stop the voice playing that note.
    ///
    /// If the vent is a global note off event, then it will stop all currently playing voices.
    pub fn handle_event(&mut self, event: &UnknownEvent) {
        match event.as_core_event() {
            Some(CoreEventSpace::NoteOn(note_event)) => {
                if let Match::Specific(key) = note_event.key() {
                    self.start_new_voice(key as u8);
                }
            }
            Some(CoreEventSpace::NoteOff(note_event)) => match note_event.key() {
                Match::All => self.stop_all(),
                Match::Specific(note_key) => self.stop_voice(note_key as u8),
            },
            _ => {}
        }
    }

    /// Generates the next batch of samples of all the currently active oscillators.
    /// Each voice will play at the given volume.
    ///
    /// This method assumes the buffer is initialized with `0`s.
    pub fn generate_next_samples(&mut self, output_buffer: &mut [f32], volume: f32) {
        for voice in &mut self.voice_buffer[..self.active_voice_count] {
            voice
                .oscillator
                .add_next_samples_to_buffer(output_buffer, volume);
        }
    }

    /// Returns `true` if any voices are currently playing, `false` otherwise.
    #[inline]
    pub fn has_active_voices(&self) -> bool {
        self.active_voice_count > 0
    }
}
