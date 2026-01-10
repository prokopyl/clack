use clack_common::utils::ClapId;
use clap_sys::ext::context_menu::*;
use core::ffi::CStr;
use std::ffi::{c_char, c_void};

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ItemKind {
    Entry = CLAP_CONTEXT_MENU_ITEM_ENTRY,
    CheckEntry = CLAP_CONTEXT_MENU_ITEM_CHECK_ENTRY,
    Separator = CLAP_CONTEXT_MENU_ITEM_SEPARATOR,
    BeginSubmenu = CLAP_CONTEXT_MENU_ITEM_BEGIN_SUBMENU,
    EndSubmenu = CLAP_CONTEXT_MENU_ITEM_END_SUBMENU,
    Title = CLAP_CONTEXT_MENU_ITEM_TITLE,
}

impl ItemKind {
    #[inline]
    pub const fn to_raw(self) -> clap_context_menu_item_kind {
        self as u32
    }

    #[inline]
    pub const fn from_raw(raw: clap_context_menu_item_kind) -> Option<Self> {
        match raw {
            CLAP_CONTEXT_MENU_ITEM_ENTRY => Some(Self::Entry),
            CLAP_CONTEXT_MENU_ITEM_CHECK_ENTRY => Some(Self::CheckEntry),
            CLAP_CONTEXT_MENU_ITEM_SEPARATOR => Some(Self::Separator),
            CLAP_CONTEXT_MENU_ITEM_BEGIN_SUBMENU => Some(Self::BeginSubmenu),
            CLAP_CONTEXT_MENU_ITEM_END_SUBMENU => Some(Self::EndSubmenu),
            CLAP_CONTEXT_MENU_ITEM_TITLE => Some(Self::Title),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Item<'a> {
    Entry {
        label: &'a CStr,
        enabled: bool,
        action_id: ClapId,
    },
    CheckEntry {
        label: &'a CStr,
        enabled: bool,
        checked: bool,
        action_id: ClapId,
    },
    Separator,
    BeginSubmenu {
        label: &'a CStr,
        enabled: bool,
    },
    EndSubmenu,
    Title {
        title: &'a CStr,
        enabled: bool,
    },
}

impl Item<'_> {
    #[inline]
    pub const fn kind(&self) -> ItemKind {
        match self {
            Item::Entry { .. } => ItemKind::Entry,
            Item::CheckEntry { .. } => ItemKind::CheckEntry,
            Item::Separator => ItemKind::Separator,
            Item::BeginSubmenu { .. } => ItemKind::BeginSubmenu,
            Item::EndSubmenu => ItemKind::EndSubmenu,
            Item::Title { .. } => ItemKind::Title,
        }
    }

    #[inline]
    pub(crate) unsafe fn from_raw(
        kind: clap_context_menu_item_kind,
        data: *const c_void,
    ) -> Option<Self> {
        const unsafe fn cstr<'a>(ptr: *const c_char) -> Option<&'a CStr> {
            if ptr.is_null() {
                None
            } else {
                // SAFETY: upheld by caller
                Some(unsafe { CStr::from_ptr(ptr) })
            }
        }

        // TODO
        match kind {
            CLAP_CONTEXT_MENU_ITEM_ENTRY => {
                let data = data.cast::<clap_context_menu_entry>().as_ref()?;
                Some(Self::Entry {
                    enabled: data.is_enabled,
                    action_id: ClapId::from_raw(data.action_id)?,
                    label: cstr(data.label)?,
                })
            }
            _ => None,
        }
    }

    #[inline]
    pub(crate) const fn raw_item(&self) -> Option<RawContextMenuItem> {
        match self {
            Item::Entry {
                label,
                enabled,
                action_id,
            } => Some(RawContextMenuItem {
                entry: clap_context_menu_entry {
                    action_id: action_id.get(),
                    is_enabled: *enabled,
                    label: label.as_ptr(),
                },
            }),
            Item::CheckEntry {
                label,
                enabled,
                checked,
                action_id,
            } => Some(RawContextMenuItem {
                check: clap_context_menu_check_entry {
                    action_id: action_id.get(),
                    is_enabled: *enabled,
                    is_checked: *checked,
                    label: label.as_ptr(),
                },
            }),
            Item::BeginSubmenu { label, enabled } => Some(RawContextMenuItem {
                submenu: clap_context_menu_submenu {
                    label: label.as_ptr(),
                    is_enabled: *enabled,
                },
            }),
            Item::Title { title, enabled } => Some(RawContextMenuItem {
                title: clap_context_menu_item_title {
                    title: title.as_ptr(),
                    is_enabled: *enabled,
                },
            }),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) union RawContextMenuItem {
    entry: clap_context_menu_entry,
    check: clap_context_menu_check_entry,
    submenu: clap_context_menu_submenu,
    title: clap_context_menu_item_title,
}
