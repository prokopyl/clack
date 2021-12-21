use crate::instance::shared::PluginInstanceShared;
use basedrop::Shared;
use std::pin::Pin;

pub(crate) enum PluginChannelMessageInner {
    Deactivate(Pin<Shared<PluginInstanceShared>>),
}

pub struct PluginChannelMessage {
    pub(crate) inner: PluginChannelMessageInner,
}

pub trait PluginInstanceChannelSend: Sized {
    fn send_message(&mut self, message: PluginChannelMessage);
}

impl<F: FnMut(PluginChannelMessage)> PluginInstanceChannelSend for F {
    #[inline]
    fn send_message(&mut self, message: PluginChannelMessage) {
        self(message)
    }
}
