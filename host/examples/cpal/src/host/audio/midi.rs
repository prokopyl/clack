use crate::host::CpalHost;
use clack_extensions::note_ports::{NoteDialects, NotePortInfoBuffer, PluginNotePorts};
use clack_host::events::event_types::{
    MidiEvent, NoteChokeEvent, NoteEvent, NoteOffEvent, NoteOnEvent,
};
use clack_host::events::EventFlags;
use clack_host::prelude::*;
use midir::{Ignore, MidiInput, MidiInputConnection};
use rtrb::{Consumer, RingBuffer};
use std::error::Error;
use wmidi::{MidiMessage, Velocity};

/// A MIDI message that was received at a given time.
struct MidiEventMessage {
    /// A micro-timestamp of when the event occurred.
    ///
    /// This is given by `midir` and is unrelated to the audio frame counter. It is based off an
    /// arbitrary start time. The only guarantee is that this timestamp is steadily increasing.
    timestamp: u64,
    /// The MIDI event. This is 'static to make it simpler to share across threads, meaning we
    /// don't support MIDI SysEx messages.
    midi_event: MidiMessage<'static>,
}

/// A receiver for the MIDI event stream.
///
/// This is to be held by the audio thread, and will collect events from the MIDI thread.
pub struct MidiReceiver {
    /// The input connection to the MIDI device.
    /// This isn't used directly, but must be kept alive to ensure keep the connection open.
    _connection: MidiInputConnection<()>,
    /// The consumer side of the ring buffer the MIDI thread sends event through.
    consumer: Consumer<MidiEventMessage>,
    /// Whether or not the ringbuffer has already been abandoned by the MIDI thread, i.e. the
    /// connection unexpectedly closed.
    ///
    /// This is used to shut down all notes when a device is disconnected.
    abandoned: bool,
    /// The buffer holding CLAP events to be fed to the plugin.
    clap_events_buffer: EventBuffer,
    /// The audio sample rate. This is used to calculate the event's sample time from the device
    /// timestamp.
    sample_rate: u64,
    /// The index of the note port the plugin uses.
    /// This is determined from the CLAP note ports extension.
    main_plugin_note_port_index: u32,
    /// If the plugin prefers to receive note events as MIDI events instead of CLAP Note events.
    prefers_midi: bool,
}

impl MidiReceiver {
    /// Connects to a MIDI device and starts receiving events.
    ///
    /// This selects the last MIDI device that was plugged in, if any.
    pub fn new(
        sample_rate: u64,
        instance: &mut PluginInstance<CpalHost>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        let Some((main_plugin_note_port_index, prefers_midi)) = find_main_note_port_index(instance)
        else {
            println!("Plugin does not have any Note inputs. It will not be fed any MIDI input.");
            return Ok(None);
        };

        let mut input = MidiInput::new("Clack Host")?;
        input.ignore(Ignore::Sysex | Ignore::Time | Ignore::ActiveSense);

        let ports = input.ports();

        if ports.is_empty() {
            println!("No MIDI input device found. Plugin will not be fed any MIDI input.");
            return Ok(None);
        }

        // PANIC: we checked ports wasn't empty above
        let selected_port = ports.last().unwrap();
        let port_name = input.port_name(selected_port)?;

        if ports.len() > 1 {
            println!("Found multiple MIDI input ports:");
            for x in &ports {
                let Ok(port_name) = input.port_name(x) else {
                    continue;
                };
                println!("\t > {port_name}")
            }

            println!("\t * Using the latest MIDI device as input: {port_name}");
        } else {
            println!("MIDI device found! Using '{port_name}' as input.");
        }

        let (mut producer, consumer) = RingBuffer::new(128);
        let connection = input.connect(
            selected_port,
            "Clack Host MIDI input",
            move |timestamp, data, ()| {
                let Ok(midi_event) = MidiMessage::try_from(data) else {
                    return;
                };
                let Some(midi_event) = midi_event.drop_unowned_sysex() else {
                    return;
                };
                let _ = producer.push(MidiEventMessage {
                    timestamp,
                    midi_event,
                });
            },
            (),
        )?;

        Ok(Some(Self {
            clap_events_buffer: EventBuffer::with_capacity(128),
            _connection: connection,
            consumer,
            sample_rate,
            abandoned: false,
            main_plugin_note_port_index,
            prefers_midi,
        }))
    }

    /// Receives all of the MIDI events since the last call to the method.
    ///
    /// Event's timestamps are interpolated to sample time between 0 and the given sample count.
    ///
    /// This returns a Clack input event buffer handle, ready to feed to the plugin.
    pub fn receive_all_events(&mut self, sample_count: u64) -> InputEvents {
        self.clap_events_buffer.clear();

        if !self.abandoned && self.consumer.is_abandoned() {
            self.clap_events_buffer.push(&NoteChokeEvent(NoteEvent::new(
                EventHeader::new_core(0, EventFlags::IS_LIVE),
                -1,
                -1,
                -1,
                -1,
                0.0,
            )));
            self.abandoned = true;
        } else {
            let mut first_event_timestamp = None;
            while let Ok(midi_event) = self.consumer.pop() {
                let first_event_timestamp =
                    *first_event_timestamp.get_or_insert(midi_event.timestamp);

                let sample_time = micro_timestamp_to_sample_time(
                    midi_event.timestamp,
                    first_event_timestamp,
                    self.sample_rate,
                    sample_count,
                );

                push_midi_to_buffer(
                    midi_event.midi_event,
                    sample_time as u32,
                    &mut self.clap_events_buffer,
                    self.main_plugin_note_port_index,
                    self.prefers_midi,
                );
            }
        }

        self.clap_events_buffer.as_input()
    }
}

/// Interpolates the given timestamp to a sample timestamp.
///
/// This takes the first received event timestamp as sample time 0, then interpolates to a maximum
/// of sample_count, according to the given sample_rate. This ensures most MIDI events are orderly
/// distributed in the sample range.
fn micro_timestamp_to_sample_time(
    timestamp: u64,
    first_event_timestamp: u64,
    sample_rate: u64,
    sample_count: u64,
) -> u64 {
    let relative_micro_timestamp = timestamp.saturating_sub(first_event_timestamp);
    let relative_micro_sample = relative_micro_timestamp.saturating_mul(sample_rate);
    let relative_sample = relative_micro_sample.saturating_div(1_000_000);

    relative_sample.min(sample_count)
}

/// Pushes a MIDI event to the given Clack event buffer.
fn push_midi_to_buffer(
    message: MidiMessage,
    sample_time: u32,
    buffer: &mut EventBuffer,
    port_index: u32,
    prefers_midi: bool,
) {
    match message {
        MidiMessage::NoteOff(channel, note, velocity) if !prefers_midi => {
            buffer.push(&NoteOffEvent(NoteEvent::new(
                EventHeader::new_core(sample_time, EventFlags::IS_LIVE),
                -1,
                port_index as i16,
                note as i16,
                channel.index() as i16,
                u8::from(velocity) as f64 / (u8::from(Velocity::MAX) as f64),
            )))
        }
        MidiMessage::NoteOn(channel, note, velocity) if !prefers_midi => {
            buffer.push(&NoteOnEvent(NoteEvent::new(
                EventHeader::new_core(sample_time, EventFlags::IS_LIVE),
                -1,
                port_index as i16,
                note as i16,
                channel.index() as i16,
                u8::from(velocity) as f64 / (u8::from(Velocity::MAX) as f64),
            )));
        }
        m => {
            let mut buf = [0; 3];
            if m.copy_to_slice(&mut buf).is_ok() {
                buffer.push(&MidiEvent::new(
                    EventHeader::new_core(sample_time, EventFlags::IS_LIVE),
                    port_index as u16,
                    buf,
                ))
            }
        }
    }
}

/// Tries to find the ID of the main note port of a plugin, and whether it supports CLAP note events
/// or not.
///
/// This returns `None` if it couldn't find one.
fn find_main_note_port_index(instance: &mut PluginInstance<CpalHost>) -> Option<(u32, bool)> {
    let handle = instance.main_thread_plugin_data();
    let plugin_note_ports = handle.shared().get_extension::<PluginNotePorts>()?;

    let mut buffer = NotePortInfoBuffer::new();
    let ports_count = plugin_note_ports.count(&handle, true);
    for i in 0..ports_count {
        let Some(port_info) = plugin_note_ports.get(&handle, i, true, &mut buffer) else {
            continue;
        };

        if !port_info
            .supported_dialects
            .intersects(NoteDialects::CLAP | NoteDialects::MIDI)
        {
            continue;
        }

        let prefers_midi = !port_info.supported_dialects.intersects(NoteDialects::CLAP);
        let port_name = String::from_utf8_lossy(port_info.name);
        println!(
            "Found Note port '{}' (ID {}, Supports CLAP events: {})",
            &port_name, port_info.id, !prefers_midi
        );

        return Some((port_info.id, prefers_midi));
    }

    None
}
