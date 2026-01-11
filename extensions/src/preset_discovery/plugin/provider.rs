use crate::preset_discovery::prelude::*;
use clack_plugin::prelude::PluginError;
pub use instance::ProviderInstance;

mod instance;

/// A provider implementation.
///
/// A provider has one main job: to give the host a list of preset metadata for a given [`Location`].
/// The host is informed about which locations to scan via
/// [`Indexer::declare_location`](Indexer::declare_location).
pub trait ProviderImpl<'a>: 'a {
    /// Gathers the metadata about presets stored at a specific given location, and gives it to the
    /// given `receiver`.
    ///
    /// The given `location` here is different from the one given to
    /// [`Indexer::declare_location`](Indexer::declare_location): here a path
    /// points directly to a file discovered by the host, instead of a directory path.
    ///
    /// The host discovered that file by recursively traversing a [`Location`] given to the
    /// [`Indexer`] (optionally filtered with [`FileType`]).
    ///
    /// # Errors
    ///
    /// If metadata fetching fails for any reason, a [`PluginError`] may be returned.
    ///
    /// # Example
    ///
    /// ```
    /// use std::ffi::{CStr, CString};
    /// use clack_common::utils::UniversalPluginId;
    /// use clack_extensions::preset_discovery::prelude::*;
    /// use clack_plugin::prelude::*;
    ///
    /// struct MyPresetProvider;
    ///
    /// impl ProviderImpl<'_> for MyPresetProvider {
    ///  fn get_metadata(&mut self, location: Location, receiver: &mut MetadataReceiver) -> Result<(), PluginError> {
    ///    match location {
    ///      // Presets that are built-in, i.e. statically contained in the plugin's file.
    ///      Location::Plugin => {
    ///         // We can use a custom load_key to differentiate between presets in the same container.
    ///         receiver.begin_preset(Some(c"Included preset 1"), Some(c"1"))?
    ///             .add_creator(c"Me!")
    ///             .add_plugin_id(UniversalPluginId::clap(c"org.example.plugin-gain-example"));
    ///
    ///         receiver.begin_preset(Some(c"Included preset 2"), Some(c"2"))?
    ///             .add_creator(c"Also me!")
    ///             .add_plugin_id(UniversalPluginId::clap(c"org.example.plugin-gain-example"));
    ///      }
    ///      Location::File { path } => {
    ///         // Parse the preset file (which can be your custom preset format)
    ///         let name = parse_preset_name(path);
    ///
    ///         // Here our format only contains one preset per file. No need for a load_key to differentiate them.
    ///         receiver.begin_preset(Some(&name), None)?
    ///             .add_creator(c"Me!")
    ///             .add_plugin_id(UniversalPluginId::clap(c"org.example.plugin-gain-example"));
    ///       }
    ///     }
    ///     Ok(())
    ///   }
    /// }
    ///
    /// fn parse_preset_name(path: &CStr) -> CString {
    /// # unreachable!()
    /// /* ... */
    /// }
    /// ```
    fn get_metadata(
        &mut self,
        location: Location,
        receiver: &mut MetadataReceiver,
    ) -> Result<(), PluginError>;
}
