use clap_sys::entry::clap_plugin_entry;

/// Entry point into the plugin. This is the interface by which the host
/// can access all of the plugin's functionality.
pub type PluginEntryDescriptor = clap_plugin_entry;
