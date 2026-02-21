use clap_sys::entry::clap_plugin_entry;

/// A C-FFI compatible descriptor of a CLAP file's entry point.
///
/// This type is what is statically exposed in the plugin's dynamic library file, and is
/// loaded by the host upon loading the file.
///
/// This type is what is exposed by the `clack-plugin` crate, and can be loaded using the
/// `clack-host` crate.
pub type EntryDescriptor = clap_plugin_entry;
