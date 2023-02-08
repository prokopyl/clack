use clap_sys::entry::clap_plugin_entry;

/// A raw plugin entry descriptor.
///
/// This type is what is statically exposed in the plugin bundle's dynamic library file, and is
/// loaded by host upon loading the file.
pub type PluginEntryDescriptor = clap_plugin_entry;
