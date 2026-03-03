use super::*;
use clack_plugin::extensions::prelude::*;
use clap_sys::id::clap_id;

impl HostContextMenu {
    /// Asks the host to populate the given `builder`, with the contents of a context menu
    /// that targets the given `target`.
    #[inline]
    pub fn populate(
        &self,
        host: &mut HostMainThreadHandle,
        target: ContextMenuTarget,
        builder: &mut ContextMenuBuilder,
    ) -> Result<(), ContextMenuError> {
        let Some(populate) = host.use_extension(&self.0).populate else {
            return Err(ContextMenuError::Builder);
        };

        let target = target.to_raw();

        // SAFETY: this type ensures the function pointer is valid.
        // All three pointers are valid for the duration of the call.
        let success = unsafe { populate(host.as_raw(), &target, builder.as_raw_mut()) };

        if success {
            Ok(())
        } else {
            Err(ContextMenuError::Builder)
        }
    }

    /// Asks the host to perform a context menu action, designated by the given `action_id`.
    ///
    /// The given `action_id` belongs to the menu created by [`populate`](Self::populate) with the
    /// given `target`.
    #[inline]
    pub fn perform(
        &self,
        host: &mut HostMainThreadHandle,
        target: ContextMenuTarget,
        action_id: ClapId,
    ) -> Result<(), ContextMenuError> {
        let Some(perform) = host.use_extension(&self.0).perform else {
            return Err(ContextMenuError::ActionFailed);
        };

        let target = target.to_raw();

        // SAFETY: this type ensures the function pointer is valid.
        // Both pointers are valid for the duration of the call.
        let success = unsafe { perform(host.as_raw(), &target, action_id.get()) };

        if success {
            Ok(())
        } else {
            Err(ContextMenuError::ActionFailed)
        }
    }

    /// Returns `true` if the host can pop up its context menu on behalf of the plugin, `false` otherwise.
    #[inline]
    pub fn can_popup(&self, host: &mut HostMainThreadHandle) -> bool {
        let Some(can_popup) = host.use_extension(&self.0).can_popup else {
            return false;
        };

        // SAFETY: this type ensures the function pointer is valid.
        // The host pointer is also guaranteed to be valid for the duration of the call.
        unsafe { can_popup(host.as_raw()) }
    }

    /// Asks the host to pop up its context menu at a given location.
    #[inline]
    pub fn popup(
        &self,
        host: &mut HostMainThreadHandle,
        target: ContextMenuTarget,
        screen_index: i32,
        x: i32,
        y: i32,
    ) -> Result<(), ContextMenuError> {
        let Some(popup) = host.use_extension(&self.0).popup else {
            return Err(ContextMenuError::Popup);
        };

        let target = target.to_raw();

        // SAFETY: this type ensures the function pointer is valid.
        // The host pointer is also guaranteed to be valid for the duration of the call.
        let success = unsafe { popup(host.as_raw(), &target, screen_index, x, y) };

        if success {
            Ok(())
        } else {
            Err(ContextMenuError::ActionFailed)
        }
    }
}

/// Implementation of the plugin-side of the Context Menu extension.
pub trait PluginContextMenuImpl {
    /// Asks the plugin to populate the given `builder`, with the contents of a context menu
    /// that targets the given `target`.
    fn populate(
        &mut self,
        target: ContextMenuTarget,
        builder: &mut ContextMenuBuilder,
    ) -> Result<(), PluginError>;

    /// Asks the plugin to perform a context menu action, designated by the given `action_id`.
    ///
    /// The given `action_id` belongs to the menu created by [`populate`](Self::populate) with the
    /// given `target`.
    fn perform(&mut self, target: ContextMenuTarget, action_id: ClapId) -> Result<(), PluginError>;
}

// SAFETY: clap_plugin_context_menu is #[repr(C)] and is the plugin-side of the Context Menu extension
unsafe impl<P> ExtensionImplementation<P> for PluginContextMenu
where
    P: for<'a> Plugin<MainThread<'a>: PluginContextMenuImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_context_menu {
            perform: Some(perform::<P>),
            populate: Some(populate::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn populate<P>(
    plugin: *const clap_plugin,
    target: *const clap_context_menu_target,
    builder: *const clap_context_menu_builder,
) -> bool
where
    P: for<'a> Plugin<MainThread<'a>: PluginContextMenuImpl>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        // SAFETY: The CLAP spec requires this pointer to be either NULL or valid for reads.
        let target = unsafe { ContextMenuTarget::from_raw_ptr(target) };

        // SAFETY: the CLAP spec requires the builder pointer and all its fields to be valid
        // for the duration of this function call, which is the (inferred) lifetime we give it here.
        let mut builder = unsafe { ContextMenuBuilder::from_raw(builder) };

        plugin
            .main_thread()
            .as_mut()
            .populate(target, &mut builder)?;

        Ok(())
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn perform<P>(
    plugin: *const clap_plugin,
    target: *const clap_context_menu_target,
    action_id: clap_id,
) -> bool
where
    P: for<'a> Plugin<MainThread<'a>: PluginContextMenuImpl>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        // SAFETY: The CLAP spec requires this pointer to be either NULL or valid for reads.
        let target = unsafe { ContextMenuTarget::from_raw_ptr(target) };

        let action_id = ClapId::from_raw(action_id)
            .ok_or(PluginWrapperError::InvalidParameter("Invalid Action ID"))?;

        plugin.main_thread().as_mut().perform(target, action_id)?;
        Ok(())
    })
    .is_some()
}
