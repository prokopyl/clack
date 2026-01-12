use crate::audio_ports::{AudioPortInfo, HostAudioPorts, PluginAudioPorts, RescanType};
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
use clap_sys::ext::audio_ports::{clap_audio_port_info, clap_plugin_audio_ports};
use std::mem::MaybeUninit;

pub struct AudioPortInfoWriter<'a> {
    buf: &'a mut MaybeUninit<clap_audio_port_info>,
    is_set: bool,
}

impl AudioPortInfoWriter<'_> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is aligned and points to a valid allocation.
    /// However, it doesn't have to be initialized.
    #[inline]
    pub(crate) unsafe fn from_raw(raw: *mut clap_audio_port_info) -> Self {
        Self {
            buf: &mut *raw.cast(),
            is_set: false,
        }
    }

    #[inline]
    pub(crate) fn is_set(&self) -> bool {
        self.is_set
    }

    #[inline]
    pub fn set(&mut self, data: &AudioPortInfo) {
        use core::ptr::write;

        let buf = self.buf.as_mut_ptr();

        // SAFETY: all pointers come from `buf`, which is valid for writes and well-aligned
        unsafe {
            write(&raw mut (*buf).id, data.id.get());
            write_to_array_buf(&raw mut (*buf).name, data.name);

            write(&raw mut (*buf).flags, data.flags.bits());
            write(&raw mut (*buf).channel_count, data.channel_count);

            write(
                &raw mut (*buf).port_type,
                data.port_type
                    .map(|s| s.0.as_ptr())
                    .unwrap_or(core::ptr::null()),
            );

            write(
                &raw mut (*buf).in_place_pair,
                ClapId::optional_to_raw(data.in_place_pair),
            );
        }

        self.is_set = true;
    }
}

/// Describes the plugin’s audio ports.
///
/// Hosts use this to discover how many input/output ports exist and to retrieve
/// metadata about each port. Information is reported through the provided
/// [`AudioPortInfoWriter`], which corresponds to
/// [`clap_plugin_audio_ports.get()`](https://github.com/free-audio/clap/blob/29ffcc273b/include/clap/ext/audio-ports.h#L76)
/// in the CLAP API.
///
/// # Example
/// Declaring a plugin with one stereo input and one stereo output:
/// ```no_run
/// use clack_plugin::audio_ports::{PluginAudioPortsImpl, AudioPortInfoWriter};
/// struct MyPluginMainThread;
///
/// impl PluginAudioPortsImpl for MyPluginMainThread {
///     fn count(&mut self, is_input: bool) -> u32 {
///         if is_input { 1 } else { 1 }
///     }
///
///     fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
///         if index == 0 {
///             writer.set(&AudioPortInfo {
///                 id: ClapId::new(0),
///                 name: match is_input {
///                     true => b"Stereo In",
///                     false => b"Stereo Out",
///                 },
///                 channel_count: 2,
///                 flags: AudioPortFlags::IS_MAIN,
///                 port_type: Some(AudioPortType::STEREO),
///                 in_place_pair: None,
///             });
///         }
///     }
/// }
/// ```
pub trait PluginAudioPortsImpl {
    /// Returns the number of audio ports for the given direction.
    ///
    /// The `is_input` flag tells you whether to report an input or output port,
    /// so implementations typically branch on it.
    fn count(&mut self, is_input: bool) -> u32;

    /// Populates metadata about the audio port at `index`.
    ///
    /// This is where you describe the port’s layout and capabilities
    /// (name, channel count, flags, etc.) using the [`AudioPortInfoWriter`].
    ///
    /// The `is_input` flag tells you whether to report an input or output port,
    /// so implementations typically branch on it.
    ///
    /// The host will call this after [`count`] to enumerate all ports.
    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginAudioPorts
where
    for<'a> P: Plugin<MainThread<'a>: PluginAudioPortsImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_audio_ports {
            count: Some(count::<P>),
            get: Some(get::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn count<P>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    for<'a> P: Plugin<MainThread<'a>: PluginAudioPortsImpl>,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_mut().count(is_input)))
        .unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get<P>(
    plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginAudioPortsImpl>,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if info.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_audio_port_info"));
        };

        let mut writer = AudioPortInfoWriter::from_raw(info);
        p.main_thread().as_mut().get(index, is_input, &mut writer);
        Ok(writer.is_set())
    })
    .unwrap_or(false)
}

impl HostAudioPorts {
    #[inline]
    pub fn is_rescan_flag_supported(&self, host: &HostMainThreadHandle, flag: RescanType) -> bool {
        match host.use_extension(&self.0).is_rescan_flag_supported {
            None => false,
            // SAFETY: This type ensures the function pointer is valid.
            Some(supported) => unsafe { supported(host.as_raw(), flag.bits()) },
        }
    }

    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle, flag: RescanType) {
        if let Some(rescan) = host.use_extension(&self.0).rescan {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { rescan(host.as_raw(), flag.bits()) }
        }
    }
}
