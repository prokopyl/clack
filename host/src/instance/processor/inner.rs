use crate::instance::channel::{
    PluginChannelMessage, PluginChannelMessageInner, PluginInstanceChannelSend,
};
use crate::instance::shared::PluginInstanceShared;
use basedrop::Shared;
use clap_sys::process::clap_process;
use std::pin::Pin;

pub(crate) struct PluginAudioProcessorInner<TChannel: PluginInstanceChannelSend> {
    shared: Option<Pin<Shared<PluginInstanceShared>>>,
    channel_send: TChannel,
}

impl<TChannel: PluginInstanceChannelSend> PluginAudioProcessorInner<TChannel> {
    pub fn new(shared: Pin<Shared<PluginInstanceShared>>, channel_send: TChannel) -> Self {
        Self {
            shared: Some(shared),
            channel_send,
        }
    }

    #[inline]
    fn shared(&self) -> &PluginInstanceShared {
        self.shared.as_ref().expect("Plugin audio processor attempted to be deactivated twice. This is a bug in clap-host, or worse")
    }

    #[inline]
    pub unsafe fn start_processing(&mut self) -> bool {
        let instance = self.shared().instance();

        (instance.start_processing)(instance)
    }

    #[inline]
    pub unsafe fn stop_processing(&mut self) {
        let instance = self.shared().instance();
        (instance.stop_processing)(instance)
    }

    #[inline]
    pub unsafe fn process(&mut self, process: &clap_process) {
        let instance = self.shared().instance();
        (instance.process)(instance, process); // TODO: handle return value
    }

    /*
    #[inline]
    pub unsafe fn send_message_to_main_thead(&mut self, message: PluginChannelMessage) {
        self.channel_send.send_message(message)
    }*/
}

impl<TChannel: PluginInstanceChannelSend> Drop for PluginAudioProcessorInner<TChannel> {
    #[inline]
    fn drop(&mut self) {
        if let Some(shared) = self.shared.take() {
            self.channel_send.send_message(PluginChannelMessage {
                inner: PluginChannelMessageInner::Deactivate(shared),
            })
        } else {
            eprintln!("Plugin audio processor attempted to be deactivated twice. This is a bug in clap-host, or worse")
        }
    }
}
