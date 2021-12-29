use crate::host::PluginHost;
use basedrop::Shared;
use clap_sys::plugin::clap_plugin;
use std::ops::RangeInclusive;
use std::pin::Pin;

use crate::entry::PluginEntry;
use crate::instance::channel::{
    PluginChannelMessage, PluginChannelMessageInner, PluginInstanceChannelSend,
};
use crate::instance::processor::inner::PluginAudioProcessorInner;
use crate::instance::processor::StoppedPluginAudioProcessor;
use crate::instance::shared::{PluginInstanceCollector, PluginInstanceShared};

mod shared;

pub mod channel;

pub struct PluginAudioConfiguration {
    pub sample_rate: f64,
    pub frames_count_range: RangeInclusive<u32>,
}

pub struct PluginInstance<'a> {
    shared: Pin<Shared<PluginInstanceShared>>,
    _collector: PluginInstanceCollector<'a>,
    is_active: bool,
}

pub mod processor;

impl<'a> PluginInstance<'a> {
    pub fn new(entry: &PluginEntry<'a>, plugin_id: &str, host: &PluginHost) -> Self {
        let (shared, _collector) =
            PluginInstanceShared::new(host.shared().clone(), entry, plugin_id);

        Self {
            shared,
            _collector,
            is_active: false,
        }
    }

    pub fn activate<TChannel: PluginInstanceChannelSend>(
        &mut self,
        configuration: PluginAudioConfiguration,
        channel_send: TChannel,
    ) -> Option<StoppedPluginAudioProcessor<TChannel>> {
        if self.is_active {
            return None;
        }

        unsafe {
            ((self.shared.instance()).activate)(
                self.shared.instance(),
                configuration.sample_rate,
                *configuration.frames_count_range.start(),
                *configuration.frames_count_range.end(),
            )
        };

        // TODO: activate should return bool
        /*if !result {
            panic!("Activation failed!"); // TODO
        }*/

        self.is_active = true;
        Some(StoppedPluginAudioProcessor::new(
            PluginAudioProcessorInner::new(self.shared.clone(), channel_send),
        ))
    }

    // TODO: check which instance this message could come from
    pub fn process_received_message(&mut self, message: PluginChannelMessage) {
        match message.inner {
            PluginChannelMessageInner::Deactivate(_) => {
                if self.is_active {
                    unsafe { ((self.shared.instance()).deactivate)(self.shared.instance()) }
                    self.is_active = false;
                }
            }
        }
    }

    #[inline]
    pub fn as_raw(&self) -> &clap_plugin {
        self.shared.instance()
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
