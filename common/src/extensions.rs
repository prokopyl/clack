use core::ffi::c_void;
use std::ptr::NonNull;

pub struct PluginExtension;
pub struct HostExtension;

pub trait ExtensionType: private::Sealed {}
impl ExtensionType for PluginExtension {}
impl ExtensionType for HostExtension {}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for PluginExtension {}
    impl Sealed for HostExtension {}
}

/// # Safety
/// The IDENTIFIER must match the official identifier for the given extension, otherwise
/// the extension data could be misinterpreted, and UB could occur
pub unsafe trait Extension<'a>: Sized + 'a {
    const IDENTIFIER: *const u8;
    type ExtensionType: ExtensionType;

    /// # Safety
    /// The extension pointer must be valid
    #[inline]
    unsafe fn from_extension_ptr(ptr: NonNull<c_void>) -> &'a Self {
        ptr.cast().as_ref()
    }
}

/// # Safety
/// The IDENTIFIER must match the official identifier for the given extension, otherwise
/// the extension data could be misinterpreted, and UB could occur
pub unsafe trait ExtensionDescriptor<'a, P>: Extension<'a> {
    type ExtensionInterface: 'static;

    const INTERFACE: &'static Self::ExtensionInterface;

    fn from_implementation() -> &'a Self {
        let ptr = NonNull::from(Self::INTERFACE).cast();
        unsafe { Self::from_extension_ptr(ptr) }
    }
}
