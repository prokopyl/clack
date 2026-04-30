//! Allows plugins and hosts to exchange menu items, and lets the plugin show the host context menu.

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::context_menu::*;
use clap_sys::id::CLAP_INVALID_ID;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

mod builder;
mod item;
pub use builder::*;
use clack_common::utils::ClapId;
pub use item::*;

/// The Host-side of the Context Menu extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostContextMenu(RawExtension<HostExtensionSide, clap_host_context_menu>);

// SAFETY: The given identifiers are both valid for the context menu extension.
unsafe impl Extension for HostContextMenu {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_CONTEXT_MENU, CLAP_EXT_CONTEXT_MENU_COMPAT];
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// The Plugin-side of the Context Menu extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginContextMenu(RawExtension<PluginExtensionSide, clap_plugin_context_menu>);

// SAFETY: The given identifiers are both valid for the context menu extension.
unsafe impl Extension for PluginContextMenu {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_CONTEXT_MENU, CLAP_EXT_CONTEXT_MENU_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// Possible targets of a context menu.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum ContextMenuTarget {
    /// The context menu targets the editor window in itself, or nothing in particular.
    Global,
    /// The context menu targets a specific parameter (e.g. a knob, slider, etc.).
    /// The associated [`ClapId`] is the unique ID of the parameter.
    Param(ClapId),
}

impl ContextMenuTarget {
    /// Gets the raw, C-FFI compatible representation of this [`ContextMenuTarget`].
    #[inline]
    pub const fn to_raw(self) -> clap_context_menu_target {
        match self {
            Self::Global => clap_context_menu_target {
                kind: CLAP_CONTEXT_MENU_TARGET_KIND_GLOBAL,
                id: CLAP_INVALID_ID,
            },
            Self::Param(param) => clap_context_menu_target {
                kind: CLAP_CONTEXT_MENU_TARGET_KIND_PARAM,
                id: param.get(),
            },
        }
    }

    /// Gets a [`ContextMenuTarget`] from its raw, C-FFI compatible representation.
    ///
    /// Note that invalid or unknown values are treated as [`Global`](Self::Global).
    #[inline]
    pub const fn from_raw(raw: clap_context_menu_target) -> Self {
        match (raw.kind, ClapId::from_raw(raw.id)) {
            (CLAP_CONTEXT_MENU_TARGET_KIND_PARAM, Some(id)) => Self::Param(id),
            _ => Self::Global,
        }
    }

    /// Gets a [`ContextMenuTarget`] from a pointer its raw, C-FFI compatible representation.
    ///
    /// This function reads the value without borrowing it. If you already have the value
    /// or a borrow to it, use [`ContextMenuTarget::from_raw`] instead.
    ///
    /// Note that invalid or unknown values are treated as [`Global`](Self::Global).
    /// If the given pointer is `NULL`, this also returns `Global`.
    ///
    /// # Safety
    ///
    /// The `raw` pointer must be valid for reads and well-aligned. It may however be `NULL`.
    #[inline]
    pub const unsafe fn from_raw_ptr(raw: *const clap_context_menu_target) -> Self {
        if raw.is_null() {
            return Self::Global;
        };

        // SAFETY: we check the null case above. In any other case, the caller is responsible for
        // this pointer to be valid and aligned.
        let raw = unsafe { raw.read() };

        Self::from_raw(raw)
    }
}

/// Errors that can occur when using context menus.
#[derive(Copy, Clone, Debug)]
pub enum ContextMenuError {
    /// Failed to populate the context menu.
    Builder,
    /// Failed to open the host pop-up context menu.
    Popup,
    /// A context menu action failed.
    ActionFailed,
}

impl Display for ContextMenuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextMenuError::Builder => f.write_str("Failed to populate context menu"),
            ContextMenuError::Popup => f.write_str("Failed to open host pop-up context menu"),
            ContextMenuError::ActionFailed => f.write_str("Failed to perform context menu action"),
        }
    }
}

impl Error for ContextMenuError {}
