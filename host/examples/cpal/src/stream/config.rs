use clack_extensions::audio_ports::{
    AudioPortFlags, AudioPortInfoBuffer, AudioPortType, PluginAudioPorts,
};
use clack_host::prelude::PluginMainThreadHandle;
use cpal::traits::DeviceTrait;
use cpal::{Device, SampleFormat, SupportedStreamConfigRange};
use std::cmp::Ordering;
use std::error::Error;

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
        c @ (Ordering::Less | Ordering::Greater) => return c.reverse(),
        Ordering::Equal => {}
    }

    // We favor types with the lowest score
    match sample_type_preference(first.sample_format())
        .cmp(&sample_type_preference(second.sample_format()))
    {
        c @ (Ordering::Less | Ordering::Greater) => return c,
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
    pub fn main_port(&self) -> &AudioPortInfo {
        &self.ports[self.main_port_index as usize]
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
            }],
        }
    }
}

#[derive(Clone, Debug)]
pub struct AudioPortInfo {
    pub id: Option<u32>,
    pub port_layout: AudioPortLayout,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum AudioPortLayout {
    Mono,
    Stereo,
    Unsupported { channel_count: usize },
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
                channel_count: info.channel_count as usize,
            },
        };

        // Store which port is the main one, and throw a warning if one already exists.
        if info.flags.contains(AudioPortFlags::IS_MAIN) && main_port_index.replace(i).is_some() {
            eprintln!("Warning: plugin defines multiple main ports. This shouldn't be allowed");
        }

        discovered_ports.push(AudioPortInfo {
            id: Some(info.id),
            port_layout,
        })
    }

    if discovered_ports.is_empty() {
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
