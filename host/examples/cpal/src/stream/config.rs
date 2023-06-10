use crate::host::CpalHost;
use clack_extensions::audio_ports::{
    AudioPortFlags, AudioPortInfoBuffer, AudioPortType, PluginAudioPorts,
};
use clack_host::prelude::{PluginAudioConfiguration, PluginInstance, PluginMainThreadHandle};
use cpal::traits::DeviceTrait;
use cpal::{
    BufferSize, Device, SampleFormat, SampleRate, StreamConfig, SupportedBufferSize,
    SupportedStreamConfigRange,
};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub fn find_device_best_output_configs(
    device: &Device,
) -> Result<Vec<SupportedStreamConfigRange>, Box<dyn Error>> {
    let mut output_configs: Vec<_> = device
        .supported_output_configs()?
        .filter(is_device_config_supported)
        .collect();

    output_configs.sort_by(compare_devices_configs);

    Ok(output_configs)
}

fn is_device_config_supported(config: &SupportedStreamConfigRange) -> bool {
    // We only support stereo and mono
    if config.channels() > 2 {
        return false;
    }
    if config.channels() < 1 {
        return false;
    }

    // Sample rates so bad, we don't want them
    if config.max_sample_rate().0 < 44_100 {
        return false;
    }

    // Unsupported sample formats
    if sample_type_preference(config.sample_format()) == u8::MAX {
        return false;
    }

    true
}

fn compare_devices_configs(
    first: &SupportedStreamConfigRange,
    second: &SupportedStreamConfigRange,
) -> Ordering {
    // We always favor Stereo to Mono
    match first.channels().cmp(&second.channels()) {
        o @ (Ordering::Less | Ordering::Greater) => return o.reverse(),
        Ordering::Equal => {}
    }

    // We favor types with the lowest score
    match sample_type_preference(first.sample_format())
        .cmp(&sample_type_preference(second.sample_format()))
    {
        o @ (Ordering::Less | Ordering::Greater) => return o,
        Ordering::Equal => {}
    }

    // Once we filtered out anything below 44.1kHz, we favor the smallest minimum sample rate
    // to avoid overkill ones for this example
    match first.min_sample_rate().cmp(&second.min_sample_rate()) {
        o @ (Ordering::Less | Ordering::Greater) => return o,
        Ordering::Equal => {}
    }
    // Use the default
    first.cmp_default_heuristics(second).reverse()
}

fn sample_type_preference(sample_type: SampleFormat) -> u8 {
    match sample_type {
        // Native plugin format, always preferred if available
        SampleFormat::F32 => 0,

        // Overkill, we don't support f64 internally so it will be casted down anyway
        SampleFormat::F64 => 1,

        // Similar-ish bit depths
        SampleFormat::I64 => 2,
        SampleFormat::U64 => 3,
        SampleFormat::I32 => 4,
        SampleFormat::U32 => 5,

        // Lower bit-depths
        SampleFormat::I16 => 6,
        SampleFormat::U16 => 7,
        SampleFormat::I8 => 8,
        SampleFormat::U8 => 9,
        _ => u8::MAX,
    }
}

#[derive(Clone, Debug)]
pub struct AudioPortsConfig {
    pub main_port_index: u32,
    pub ports: Vec<AudioPortInfo>,
}

impl AudioPortsConfig {
    pub fn empty() -> Self {
        AudioPortsConfig {
            main_port_index: 0,
            ports: vec![],
        }
    }

    pub fn main_port(&self) -> &AudioPortInfo {
        &self.ports[self.main_port_index as usize]
    }

    pub fn total_channel_count(&self) -> usize {
        self.ports
            .iter()
            .map(|p| p.port_layout.channel_count() as usize)
            .sum()
    }
}

/// The default port configuration, if the plugin does not implement the port extension.
impl Default for AudioPortsConfig {
    fn default() -> Self {
        AudioPortsConfig {
            main_port_index: 0,
            ports: vec![AudioPortInfo {
                id: None,
                port_layout: AudioPortLayout::Stereo,
                name: "Default".into(),
            }],
        }
    }
}

#[derive(Clone, Debug)]
pub struct AudioPortInfo {
    pub id: Option<u32>,
    pub port_layout: AudioPortLayout,
    pub name: String,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum AudioPortLayout {
    Mono,
    Stereo,
    Unsupported { channel_count: u16 },
}

impl AudioPortLayout {
    pub fn channel_count(&self) -> u16 {
        match self {
            AudioPortLayout::Mono => 1,
            AudioPortLayout::Stereo => 2,
            AudioPortLayout::Unsupported { channel_count } => *channel_count,
        }
    }
}

impl Display for AudioPortLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioPortLayout::Mono => f.write_str("mono"),
            AudioPortLayout::Stereo => f.write_str("stereo"),
            AudioPortLayout::Unsupported { channel_count } => write!(f, "{channel_count}-channels"),
        }
    }
}

pub fn find_config_from_ports(plugin: &PluginMainThreadHandle, is_input: bool) -> AudioPortsConfig {
    let Some(ports) = plugin.shared().get_extension::<PluginAudioPorts>() else {
        return AudioPortsConfig::default();
    };

    let mut buffer = AudioPortInfoBuffer::new();
    let mut main_port_index = None;
    let mut discovered_ports = vec![];

    for i in 0..ports.count(plugin, is_input) {
        let Some(info) = ports.get(plugin, i, is_input, &mut buffer) else { continue };
        // If no port type is specified, we try to assume it from the channel count
        let port_type = info
            .port_type
            .or_else(|| AudioPortType::from_channel_count(info.channel_count));

        let port_layout = match port_type {
            Some(l) if l == AudioPortType::MONO => AudioPortLayout::Mono,
            Some(l) if l == AudioPortType::STEREO => AudioPortLayout::Stereo,
            _ => AudioPortLayout::Unsupported {
                channel_count: info.channel_count as u16,
            },
        };

        // Store which port is the main one, and throw a warning if one already exists.
        if info.flags.contains(AudioPortFlags::IS_MAIN) && main_port_index.replace(i).is_some() {
            eprintln!("Warning: plugin defines multiple main ports. This shouldn't be allowed");
        }

        discovered_ports.push(AudioPortInfo {
            id: Some(info.id),
            port_layout,
            name: String::from_utf8_lossy(info.name).into_owned(),
        })
    }

    if discovered_ports.is_empty() {
        if is_input {
            return AudioPortsConfig::empty();
        }
        eprintln!("Warning: Plugin's audio port extension returned NO port at all? Using default stereo configuration instead.");
        return AudioPortsConfig::default();
    }

    let main_port_index = if let Some(main_port_index) = main_port_index {
        main_port_index
    } else {
        eprintln!("Warning: Plugin's audio port extension defines no main port! Using the first decent port as a fallback.");
        if let Some(first_stereo_port) = discovered_ports
            .iter()
            .enumerate()
            .find(|(_, p)| p.port_layout == AudioPortLayout::Stereo)
        {
            first_stereo_port.0 as u32
        } else if let Some(first_mono_port) = discovered_ports
            .iter()
            .enumerate()
            .find(|(_, p)| p.port_layout == AudioPortLayout::Mono)
        {
            first_mono_port.0 as u32
        } else {
            eprintln!("Warning: No suitable mono or stereo port found. Will do my best.");
            0 // Assume the first port is good enough, whatever it is.
        }
    };

    AudioPortsConfig {
        main_port_index,
        ports: discovered_ports,
    }
}

pub struct FullAudioConfig {
    pub plugin_input_port_config: AudioPortsConfig,
    pub plugin_output_port_config: AudioPortsConfig,
    pub output_channel_count: usize,
    pub min_buffer_size: u32,
    pub max_buffer_size: u32,
    pub sample_rate: u32,
    pub sample_format: SampleFormat,
}

impl FullAudioConfig {
    pub fn negociate_from(
        device: &Device,
        instance: &mut PluginInstance<CpalHost>,
    ) -> Result<Self, Box<dyn Error>> {
        let best = find_device_best_output_configs(device)?;

        let input_ports = find_config_from_ports(&instance.main_thread_plugin_data(), true);
        let output_ports = find_config_from_ports(&instance.main_thread_plugin_data(), false);

        Ok(find_matching_output_config(
            &best,
            output_ports,
            input_ports,
        ))
    }

    pub fn as_cpal_stream_config(&self) -> StreamConfig {
        StreamConfig {
            channels: self.output_channel_count as u16,
            buffer_size: BufferSize::Fixed(self.max_buffer_size),
            sample_rate: SampleRate(self.sample_rate),
        }
    }

    pub fn as_clack_plugin_config(&self) -> PluginAudioConfiguration {
        PluginAudioConfiguration {
            sample_rate: self.sample_rate as f64,
            frames_count_range: self.min_buffer_size..=self.max_buffer_size,
        }
    }
}

impl Display for FullAudioConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} channels at {:.1}kHz, with buffer length of {}-{}, fed from plugin's \"{}\" port ({})",
            self.output_channel_count,
            self.sample_rate as f64 / 1_000.0,
            self.min_buffer_size,
            self.max_buffer_size,
            &self.plugin_output_port_config.main_port().name,
            self.plugin_output_port_config.main_port().port_layout
        )
    }
}

pub fn find_matching_output_config(
    ordered_stream_configs: &[SupportedStreamConfigRange],
    plugin_config: AudioPortsConfig,
    input_config: AudioPortsConfig,
) -> FullAudioConfig {
    let plugin_channel_count = plugin_config.main_port().port_layout.channel_count();

    let matching_config = ordered_stream_configs
        .iter()
        .find(|c| c.channels() == plugin_channel_count);

    let best_stream_config = matching_config
        .or_else(|| ordered_stream_configs.first())
        .expect("No config supported by output device");

    let (min_buffer_size, max_buffer_size) = match best_stream_config.buffer_size() {
        SupportedBufferSize::Range { min, max } => (*min, 1024.clamp(*min, *max)),
        SupportedBufferSize::Unknown => (1, 1024),
    };

    FullAudioConfig {
        output_channel_count: best_stream_config.channels() as usize,
        min_buffer_size,
        max_buffer_size,
        sample_rate: 44_100.clamp(
            best_stream_config.min_sample_rate().0,
            best_stream_config.max_sample_rate().0,
        ),
        plugin_output_port_config: plugin_config,
        plugin_input_port_config: input_config,
        sample_format: best_stream_config.sample_format(),
    }
}
