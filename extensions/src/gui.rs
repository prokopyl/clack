//! Allows plugins to present a GUI.
//!
//! There are two available approaches, either:
//!
//! * The plugin the plugin creates a window and embeds it into the host's window. This is often the
//!   preferred option, as it gives more control to the host, and feels more integrated.
//!
//! * The plugin creates an independent, floating window. This option should always be supported, at
//!   least as a fallback, in case window embedding is not available, which can be the case due to
//!   technical limitations.
//!
//! ## Opening a Plugin GUI
//!
//! In order to open a Plugin's GUI, Hosts must perform the following steps in order:
//!
//! * Call `is_api_supported` and `get_preferred_api` to negotiate a Windowing API to use.
//! * Call `create` to instantiate and allocate the Plugin's GUI resources.
//! * Decide whether the window will be floating or embedded.
//!   * If floating:
//!     * Call `set_transient` to make sure the Plugin's window stays above the Host's.
//!     * Call `suggest_title` to set the Plugin's window title to match the Host.
//!   * If embedded:
//!     * Set the plugin's GUI scaling using `set_scale`.
//!     * Check if the plugin's GUI can be resized by calling `can_resize`:
//!       * If it is resizeable and the hosts wants to give it a specific size (e.g. from a saved
//!         previous session), call `set_size`.
//!       * Otherwise, call `get_size` to get the initial size and adjust the parent window's client
//!         area accordingly.
//!     * Call `set_parent`.
//! * Call `show`, after which the host can call `hide` and `show` at will.
//! * Call `destroy` to free the GUI resources when the Host is done with it.
//!
//! ### Resize an embedded Plugin window.
//!
//! When the users drags to resize an embedded Plugin window, the Host must follow these steps to
//! negotiate a new window size for the plugin:
//!
//! * Check if the plugin's GUI can be resized in the first place using `can_resize`. Resizing
//!   should be completely prevented if this isn't the case.
//! * Determine a given `new_size` from the user's resizing action.
//! * Adjust it to something acceptable by the plugin by calling `adjust_size(new_size)` to get
//!   new `working_size`.
//! * Once negotiated, call `set_size(working_size)` to let the plugin redraw and resize its UI.

#![deny(missing_docs)]

use clack_common::extensions::{Extension, HostExtensionType, PluginExtensionType};
use clap_sys::ext::gui::*;
use std::cmp::Ordering;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};

mod window;
pub use window::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

/// Plugin-provided hints about how to resize its window.
///
/// This only makes sense for embedded windows.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GuiResizeHints {
    /// Whether or not the window can be resized horizontally
    pub can_resize_horizontally: bool,
    /// Whether or not the window can be resized vertically
    pub can_resize_vertically: bool,

    /// How to approach Aspect Ratio preservation
    pub strategy: AspectRatioStrategy,
}

impl GuiResizeHints {
    #[cfg(feature = "clack-host")]
    // TODO: make pub?
    fn from_raw(raw: &clap_gui_resize_hints) -> Self {
        Self {
            can_resize_horizontally: raw.can_resize_horizontally,
            can_resize_vertically: raw.can_resize_vertically,
            strategy: if !(raw.can_resize_horizontally || raw.can_resize_vertically)
                || !raw.preserve_aspect_ratio
                || raw.aspect_ratio_width == 0
                || raw.aspect_ratio_height == 0
            {
                AspectRatioStrategy::Disregard
            } else {
                AspectRatioStrategy::Preserve {
                    width: raw.aspect_ratio_width,
                    height: raw.aspect_ratio_height,
                }
            },
        }
    }

    #[cfg(feature = "clack-plugin")]
    fn to_raw(self) -> clap_gui_resize_hints {
        let mut hints = clap_gui_resize_hints {
            can_resize_horizontally: self.can_resize_horizontally,
            can_resize_vertically: self.can_resize_vertically,
            preserve_aspect_ratio: false,
            aspect_ratio_width: 1,
            aspect_ratio_height: 1,
        };

        if let AspectRatioStrategy::Preserve { width, height } = self.strategy {
            hints.preserve_aspect_ratio = true;
            hints.aspect_ratio_width = width;
            hints.aspect_ratio_height = height;
        }

        hints
    }
}

/// Represent possible strategies regarding a plugin window's Aspect Ratio.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AspectRatioStrategy {
    /// The plugin doesn't care about Aspect Ratio, it can be disregarded
    Disregard,
    /// The plugin wants to preserve a given Aspect Ratio
    Preserve {
        /// The width component of the Aspect Ratio
        width: u32,
        /// The height component of the Aspect Ratio
        height: u32,
    },
}

/// The Plugin-side of the GUI extension.
#[repr(C)]
pub struct PluginGui {
    inner: clap_plugin_gui,
}

unsafe impl Extension for PluginGui {
    const IDENTIFIER: &'static CStr = CLAP_EXT_GUI;
    type ExtensionType = PluginExtensionType;
}

/// The Host-side of the GUI extension.
#[repr(C)]
pub struct HostGui {
    inner: clap_host_gui,
}

unsafe impl Extension for HostGui {
    const IDENTIFIER: &'static CStr = CLAP_EXT_GUI;
    type ExtensionType = HostExtensionType;
}

/// Errors that can occur related to Plugin GUI handling.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GuiError {
    /// The plugin failed to create its GUI.
    CreateError,
    /// The plugin failed to set its scale.
    SetScaleError,
    /// The plugin failed to set a parent window for its GUI.
    SetParentError,
    /// The plugin failed to set a transient window for its GUI.
    SetTransientError,
    /// The plugin's window could not be resized.
    ResizeError,
    /// The plugin failed to show its GUI.
    ShowError,
    /// The plugin failed to hide its GUI.
    HideError,

    /// The host denied or failed to process a request to resize its parent window.
    RequestResizeError,
    /// The host denied or failed to process a request to show its parent window.
    RequestShowError,
    /// The host denied or failed to process a request to hide its parent window.
    RequestHideError,
}

impl Display for GuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::ResizeError => f.write_str("Failed to resize plugin window"),
            GuiError::ShowError => f.write_str("Failed to show plugin window"),
            GuiError::HideError => f.write_str("Failed to hide plugin window"),
            GuiError::CreateError => f.write_str("Failed to create plugin GUI"),
            GuiError::SetScaleError => f.write_str("Failed to set plugin window scaling"),
            GuiError::SetParentError => f.write_str("Failed to set plugin window parent"),
            GuiError::SetTransientError => f.write_str("Failed to set plugin transient"),
            GuiError::RequestResizeError => {
                f.write_str("Request to resize host parent window failed")
            }
            GuiError::RequestShowError => f.write_str("Request to show host parent window failed"),
            GuiError::RequestHideError => f.write_str("Request to hide host parent window failed"),
        }
    }
}

/// The size of a given GUI Window, in pixels.
///
/// Note that the used [`GuiApiType`] is responsible to define if it is using logical or physical pixels.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GuiSize {
    /// The width of the GUI Window, in pixels.
    pub width: u32,
    /// The height of the GUI Window, in pixels.
    pub height: u32,
}

impl PartialOrd for GuiSize {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GuiSize {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.area().cmp(&other.area())
    }
}

impl GuiSize {
    /// Computes the area, in pixels, that would be covered by a rectangular window of this size.
    ///
    /// This method saturates the resulting [`u64`] if the resulting value is too large to fit.
    #[inline]
    pub fn area(self) -> u64 {
        (self.width as u64).saturating_mul(self.height as u64)
    }

    /// Packs this [`GuiSize`] into a single [`u64`] value.
    ///
    /// This may be useful in case [`u64`]s are better to store than a pair of [`u32`], e.g. with
    /// Atomics.
    ///
    /// Use [`from_u64`](GuiSize::from_u64) to unpack this value.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_extensions::gui::GuiSize;
    ///
    /// let ui_size = GuiSize { width: 42, height: 69 };
    /// let packed = ui_size.to_u64();
    ///
    /// assert_eq!(ui_size, GuiSize::from_u64(packed));
    /// ```
    #[inline]
    pub fn to_u64(self) -> u64 {
        self.width as u64 + ((self.height as u64) << u32::BITS)
    }

    /// Unpacks a single [`u64`] value into a new [`GuiSize`].
    ///
    /// Use [`to_u64`](GuiSize::to_u64) create the packed [`u64`] value.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_extensions::gui::GuiSize;
    ///
    /// let ui_size = GuiSize { width: 42, height: 69 };
    /// let packed = ui_size.to_u64();
    ///
    /// assert_eq!(ui_size, GuiSize::from_u64(packed));
    /// ```
    #[inline]
    pub fn from_u64(raw: u64) -> Self {
        GuiSize {
            width: raw as u32,
            height: (raw >> u32::BITS) as u32,
        }
    }
}

/// A type of GUI API used to display windows to the user.
///
/// This is a simple wrapper around a C string constant, which can hold custom values as well as
/// the standard-provided ones.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GuiApiType<'a>(pub &'a CStr);

impl<'a> GuiApiType<'a> {
    /// Represents the Win32 API used by Windows.
    ///
    /// This API uses physical size for pixels.
    ///
    /// See <https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent> to learn
    /// more about embedding using the Win32 API.
    pub const WIN32: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"win32\0") });

    /// Represents the Cocoa API used by MacOS.
    ///
    /// This API uses logical size for pixels.
    ///
    /// The `set_scale` method should not be called with this GUI API, as it is all handled by the
    /// OS directly.
    pub const COCOA: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"cocoa\0") });

    /// Represents the X11 API, used by various Unix OSes.
    ///
    /// This API uses physical size for pixels.
    ///
    /// See <https://specifications.freedesktop.org/xembed-spec/xembed-spec-latest.html> to learn more
    /// about embedding using the X11 API.
    pub const X11: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"x11\0") });

    /// Represents the Wayland API, used by various, newer Unix OSes.
    ///
    /// This API uses physical size for pixels.
    ///
    /// This API does *not* support embedding as of now. You can still use floating windows.
    pub const WAYLAND: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"wayland\0") });

    /// Whether or not this API type can provide a [`RawWindowHandle`](raw_window_handle::RawWindowHandle).
    pub fn can_provide_raw_window_handle(&self) -> bool {
        self == &Self::WIN32 || self == &Self::COCOA || self == &Self::X11
    }

    /// Returns the default API type for the platform this executable is compiled for.
    ///
    /// This returns [`WIN32`](Self::WIN32) on Windows, [`COCOA`](Self::COCOA) on MacOS, and
    /// [`X11`](Self::X11) on other Unix OSes.
    #[inline]
    #[allow(unreachable_code)]
    pub fn default_for_current_platform() -> Option<Self> {
        #[cfg(target_os = "windows")]
        return Some(Self::WIN32);
        #[cfg(target_os = "macos")]
        return Some(Self::COCOA);
        #[cfg(unix)]
        return Some(Self::X11);

        None
    }
}

impl<'a> Debug for GuiApiType<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.to_str() {
            Ok(s) => f.write_str(s),
            Err(_) => self.0.to_bytes().fmt(f),
        }
    }
}
