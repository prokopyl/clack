use super::*;
use crate::utils::handle_panic;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;

/// An implementation of a [`ContextMenuBuilder`].
pub trait ContextMenuBuilderImpl: Sized {
    /// Returns `true` if the given type of menu item is supported, `false` otherwise.
    fn supports(&mut self, item_type: ItemKind) -> bool;
    /// Adds the given item to the context menu.
    ///
    /// # Errors
    ///
    /// This can return [`ContextMenuError::Builder`] if the item type is not supported, or if
    /// adding the item failed for any other reason.
    fn add_item(&mut self, item: Item) -> Result<(), ContextMenuError>;
}

/// Builds a Context Menu from a list of [`Item`]s.
///
/// This type is actually a type-erased, thin wrapper around a [`ContextMenuBuilderImpl`], which
/// allows it to be passed between plugin and host.
#[repr(C)]
pub struct ContextMenuBuilder<'a> {
    raw: clap_context_menu_builder,
    _ctx: PhantomData<&'a mut ()>,
}

impl<'a> ContextMenuBuilder<'a> {
    /// Creates a new [`ContextMenuBuilder`] by wrapping a unique reference to a given implementation.
    #[inline]
    pub const fn new<I: ContextMenuBuilderImpl>(implementation: &'a mut I) -> Self {
        Self {
            _ctx: PhantomData,
            raw: clap_context_menu_builder {
                ctx: implementation as *mut I as *mut _,
                supports: Some(supports::<I>),
                add_item: Some(add_item::<I>),
            },
        }
    }

    /// Returns a unique reference to this builder's raw, C-FFI compatible representation.
    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut clap_context_menu_builder {
        &mut self.raw
    }

    /// Retrieves a [`ContextMenuBuilder`] from a given pointer to its raw, C-FFI compatible representation.
    ///
    /// # Safety
    ///
    /// The given `raw` pointer must be valid for reads, and both the context pointer `ctx` and all
    /// function pointers in the pointed `clap_context_menu_builder` must be and remain valid for `'a`.
    #[inline]
    pub unsafe fn from_raw(raw: *const clap_context_menu_builder) -> Self {
        Self {
            // SAFETY: The caller ensures the pointer is valid for reads.
            raw: unsafe { raw.read() },
            _ctx: PhantomData,
        }
    }

    /// Returns `true` if the given type of menu item is supported, `false` otherwise.
    pub fn supports(&mut self, item_kind: ItemKind) -> bool {
        let Some(supports) = self.raw.supports else {
            return false;
        };

        // SAFETY: This type ensures self.raw is valid
        unsafe { supports(&self.raw, item_kind.to_raw()) }
    }

    /// Adds the given item to the context menu.
    ///
    /// # Errors
    ///
    /// This can return [`ContextMenuError::Builder`] if the item type is not supported, or if
    /// adding the item failed for any other reason.
    pub fn add_item(&mut self, item: &Item) -> Result<(), ContextMenuError> {
        let Some(add_item) = self.raw.add_item else {
            return Err(ContextMenuError::Builder);
        };

        let raw_item = item.raw_item();
        let raw_item_ptr = if let Some(raw_item) = &raw_item {
            raw_item as *const _ as *const c_void
        } else {
            std::ptr::null()
        };

        let item_kind = item.kind();

        // SAFETY: This type ensures self.raw is valid, and raw_item_ptr is guaranteed to be valid
        // or null by raw_item().
        let success = unsafe { add_item(&self.raw, item_kind.to_raw(), raw_item_ptr) };

        if success {
            Ok(())
        } else {
            Err(ContextMenuError::Builder)
        }
    }
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn supports<I: ContextMenuBuilderImpl>(
    builder: *const clap_context_menu_builder,
    item_kind: clap_context_menu_item_kind,
) -> bool {
    handle::<I, _>(builder, |builder| {
        let item_kind = ItemKind::from_raw(item_kind)?;
        Some(builder.supports(item_kind))
    })
    .unwrap_or(false)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_item<I: ContextMenuBuilderImpl>(
    builder: *const clap_context_menu_builder,
    item_kind: clap_context_menu_item_kind,
    item_data: *const c_void,
) -> bool {
    handle::<I, _>(builder, |builder| {
        let item = Item::from_raw(item_kind, item_data)?;
        builder.add_item(item).ok()
    })
    .is_some()
}

#[allow(clippy::missing_safety_doc)]
#[inline]
unsafe fn handle<I: ContextMenuBuilderImpl, T>(
    builder: *const clap_context_menu_builder,
    handler: impl FnOnce(&mut I) -> Option<T>,
) -> Option<T> {
    handle_panic(AssertUnwindSafe(|| {
        if builder.is_null() {
            return None;
        }

        // SAFETY: builder.ctx should be in-bounds of the allocation and valid for reads
        let ctx = unsafe { (*builder).ctx };

        // SAFETY: The caller guarantees this ref is unique and of type I
        let ctx = unsafe { ctx.cast::<I>().as_mut() }?;

        handler(ctx)
    }))
    .ok()
    .flatten()
}
