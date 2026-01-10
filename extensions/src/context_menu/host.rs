use super::*;
use clack_host::extensions::prelude::*;
use clap_sys::id::clap_id;

impl PluginContextMenu {
    #[inline]
    pub fn populate(
        &self,
        plugin: &mut PluginMainThreadHandle,
        target: ContextMenuTarget,
        builder: &mut ContextMenuBuilder,
    ) -> Result<(), ContextMenuError> {
        let Some(populate) = plugin.use_extension(&self.0).populate else {
            return Err(ContextMenuError::Builder);
        };

        let target = target.to_raw();

        // SAFETY: this type ensures the function pointer is valid.
        // All three pointers are valid for the duration of the call.
        let success = unsafe { populate(plugin.as_raw(), &target, builder.as_raw_mut()) };

        if success {
            Ok(())
        } else {
            Err(ContextMenuError::Builder)
        }
    }

    #[inline]
    pub fn perform(
        &self,
        plugin: &mut PluginMainThreadHandle,
        target: ContextMenuTarget,
        action_id: ClapId,
    ) -> Result<(), ContextMenuError> {
        let Some(perform) = plugin.use_extension(&self.0).perform else {
            return Err(ContextMenuError::ActionFailed);
        };

        let target = target.to_raw();

        // SAFETY: this type ensures the function pointer is valid.
        // Both pointers are valid for the duration of the call.
        let success = unsafe { perform(plugin.as_raw(), &target, action_id.get()) };

        if success {
            Ok(())
        } else {
            Err(ContextMenuError::ActionFailed)
        }
    }
}

pub trait HostContextMenuImpl {
    fn populate(
        &mut self,
        target: ContextMenuTarget,
        builder: &mut ContextMenuBuilder,
    ) -> Result<(), HostError>;

    fn perform(&mut self, target: ContextMenuTarget, action_id: ClapId) -> Result<(), HostError>;

    fn can_popup(&mut self) -> bool;
    fn popup(
        &mut self,
        target: ContextMenuTarget,
        screen_index: i32,
        x: i32,
        y: i32,
    ) -> Result<(), HostError>;
}

// SAFETY: clap_host_context_menu is #[repr(C)] and is the host-side of the Context Menu extension
unsafe impl<H> ExtensionImplementation<H> for HostContextMenu
where
    H: for<'a> HostHandlers<MainThread<'a>: HostContextMenuImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_context_menu {
            perform: Some(perform::<H>),
            populate: Some(populate::<H>),
            can_popup: Some(can_popup::<H>),
            popup: Some(popup::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn populate<H>(
    host: *const clap_host,
    target: *const clap_context_menu_target,
    builder: *const clap_context_menu_builder,
) -> bool
where
    H: for<'a> HostHandlers<MainThread<'a>: HostContextMenuImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        // SAFETY: The CLAP spec requires this pointer to be either NULL or valid for reads.
        let target = unsafe { ContextMenuTarget::from_raw_ptr(target) };

        // SAFETY: the CLAP spec requires the builder pointer and all its fields to be valid
        // for the duration of this function call, which is the (inferred) lifetime we give it here.
        let mut builder = unsafe { ContextMenuBuilder::from_raw(builder) };

        host.main_thread().as_mut().populate(target, &mut builder)?;

        Ok(())
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn perform<H>(
    host: *const clap_host,
    target: *const clap_context_menu_target,
    action_id: clap_id,
) -> bool
where
    H: for<'a> HostHandlers<MainThread<'a>: HostContextMenuImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        // SAFETY: The CLAP spec requires this pointer to be either NULL or valid for reads.
        let target = unsafe { ContextMenuTarget::from_raw_ptr(target) };

        let action_id = ClapId::from_raw(action_id)
            .ok_or(HostWrapperError::InvalidParameter("Invalid Action ID"))?;

        host.main_thread().as_mut().perform(target, action_id)?;
        Ok(())
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn can_popup<H>(host: *const clap_host) -> bool
where
    H: for<'a> HostHandlers<MainThread<'a>: HostContextMenuImpl>,
{
    HostWrapper::<H>::handle(host, |host| Ok(host.main_thread().as_mut().can_popup()))
        .unwrap_or(false)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn popup<H>(
    host: *const clap_host,
    target: *const clap_context_menu_target,
    screen_index: i32,
    x: i32,
    y: i32,
) -> bool
where
    H: for<'a> HostHandlers<MainThread<'a>: HostContextMenuImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        // SAFETY: The CLAP spec requires this pointer to be either NULL or valid for reads.
        let target = unsafe { ContextMenuTarget::from_raw_ptr(target) };
        host.main_thread()
            .as_mut()
            .popup(target, screen_index, x, y)?;
        Ok(())
    })
    .is_some()
}
