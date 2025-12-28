use clack_extensions::preset_discovery::indexer::Indexer;
use clack_extensions::preset_discovery::{Flags, Location, MetadataReceiver, Provider};
use clack_host::utils::{Timestamp, UniversalPluginID};
use core::ffi::CStr;
use std::fmt::{Display, Formatter};

pub fn get_metadata(
    provider: &mut Provider<impl Indexer>,
    location: Location,
) -> Result<Vec<PresetData>, MetadataError> {
    let mut receiver = MyMetadataReceiver::new();
    provider.get_metadata(location, &mut receiver);

    receiver.into_presets()
}

#[derive(Debug)]
pub struct PluginId {
    abi: Box<CStr>,
    id: Box<CStr>,
}

#[derive(Default, Debug)]
pub struct PresetData {
    name: Option<Box<CStr>>,
    load_key: Option<Box<CStr>>,
    plugin_ids: Vec<PluginId>,
    flags: Flags,
    soundpack_id: Option<Box<CStr>>,
    creators: Vec<Box<CStr>>,
    description: Option<Box<CStr>>,
    creation_time: Option<Timestamp>,
    modification_time: Option<Timestamp>,
    features: Vec<Box<CStr>>,
    extra_infos: Vec<(Box<CStr>, Box<CStr>)>,
}

impl Display for PresetData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{name:?}")?;
        }

        Ok(())
        //todo!()
    }
}

struct MyMetadataReceiver {
    presets: Vec<PresetData>,
    current_preset: Option<PresetData>,
    error: Option<MetadataError>,
}

impl MyMetadataReceiver {
    pub fn new() -> Self {
        Self {
            presets: vec![],
            current_preset: None,
            error: None,
        }
    }

    pub fn into_presets(mut self) -> Result<Vec<PresetData>, MetadataError> {
        if let Some(error) = self.error {
            return Err(error);
        }

        if let Some(preset) = self.current_preset {
            self.presets.push(preset);
        };

        Ok(self.presets)
    }
}

impl MetadataReceiver for MyMetadataReceiver {
    fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>) {
        self.error = Some(MetadataError {
            code: error_code,
            message: error_message.map(|c| c.to_owned().into_boxed_c_str()),
        })
    }

    fn begin_preset(&mut self, name: Option<&CStr>, load_key: Option<&CStr>) {
        if let Some(current_preset) = self.current_preset.take() {
            self.presets.push(current_preset);
        }

        self.current_preset = Some(PresetData {
            name: name.map(|s| s.to_owned().into_boxed_c_str()),
            load_key: load_key.map(|s| s.to_owned().into_boxed_c_str()),
            ..PresetData::default()
        })
    }

    fn add_plugin_id(&mut self, plugin_id: UniversalPluginID) {
        self.current_preset
            .get_or_insert_default()
            .plugin_ids
            .push(PluginId {
                abi: plugin_id.abi.to_owned().into_boxed_c_str(),
                id: plugin_id.id.to_owned().into_boxed_c_str(),
            })
    }

    fn set_soundpack_id(&mut self, soundpack_id: &CStr) {
        self.current_preset.get_or_insert_default().soundpack_id =
            Some(soundpack_id.to_owned().into_boxed_c_str());
    }

    fn set_flags(&mut self, flags: Flags) {
        self.current_preset.get_or_insert_default().flags = flags;
    }

    fn add_creator(&mut self, creator: &CStr) {
        self.current_preset
            .get_or_insert_default()
            .creators
            .push(creator.to_owned().into_boxed_c_str());
    }

    fn set_description(&mut self, description: &CStr) {
        self.current_preset.get_or_insert_default().description =
            Some(description.to_owned().into_boxed_c_str());
    }

    fn set_timestamps(
        &mut self,
        creation_time: Option<Timestamp>,
        modification_time: Option<Timestamp>,
    ) {
        let current_preset = self.current_preset.get_or_insert_default();
        current_preset.creation_time = creation_time;
        current_preset.modification_time = modification_time;
    }

    fn add_feature(&mut self, feature: &CStr) {
        self.current_preset
            .get_or_insert_default()
            .features
            .push(feature.to_owned().into_boxed_c_str());
    }

    fn add_extra_info(&mut self, key: &CStr, value: &CStr) {
        self.current_preset
            .get_or_insert_default()
            .extra_infos
            .push((
                key.to_owned().into_boxed_c_str(),
                value.to_owned().into_boxed_c_str(),
            ));
    }
}

#[derive(Debug)]
pub struct MetadataError {
    code: i32,
    message: Option<Box<CStr>>,
}

impl Display for MetadataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.code, &self.message) {
            (0, None) => write!(f, "Error while receiving metadata"),
            (0, Some(message)) => write!(f, "{}", message.to_string_lossy()),
            (code, None) => write!(f, "Error while receiving metadata (Error code {code})"),
            (code, Some(message)) => write!(f, "{} (Error code {code})", message.to_string_lossy()),
        }
    }
}

impl std::error::Error for MetadataError {}
