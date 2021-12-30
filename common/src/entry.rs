use clap_sys::plugin::clap_plugin_entry;

// TODO: this doesn't look very useful
#[repr(C)]
pub struct PluginEntryDescriptor(clap_plugin_entry);

impl PluginEntryDescriptor {
    #[inline]
    pub const fn new(raw: clap_plugin_entry) -> Self {
        Self(raw)
    }

    #[inline]
    pub fn from_raw(raw: &clap_plugin_entry) -> &Self {
        unsafe { ::core::mem::transmute(raw) }
    }

    #[inline]
    pub fn as_raw(&self) -> &clap_plugin_entry {
        &self.0
    }
}
