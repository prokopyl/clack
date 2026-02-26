use crate::utils::cstr_from_nullable_ptr;
use clack_common::utils::ClapId;
use clap_sys::ext::context_menu::*;
use core::ffi::CStr;
use std::ffi::c_void;

/// The kinds of Context Menu [`Item`]s.
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
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

/// Items that may be present in a context menu.
///
/// Many items borrow string references for their labels, which need to remain valid for the `'a` lifetime.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Item<'a> {
    /// A simple context menu entry that can be activated.
    Entry {
        /// The label of the menu entry.
        label: &'a CStr,
        /// Whether the entry is enabled.
        enabled: bool,
        /// The ID of the action that is triggered when this entry is activated.
        action_id: ClapId,
    },
    /// A context menu entry with a check-box.
    CheckEntry {
        /// The label of the menu entry.
        label: &'a CStr,
        /// Whether the entry is enabled.
        enabled: bool,
        /// Whether the check-box of the entry is checked or not.
        checked: bool,
        /// The ID of the action that is triggered when this entry is activated.
        action_id: ClapId,
    },
    /// A menu separator.
    Separator,
    /// Marks the beginning of a submenu.
    ///
    /// All further items after this will be associated to this submenu.
    BeginSubmenu { label: &'a CStr, enabled: bool },
    /// Marks the end of a submenu.
    ///
    /// All further items after this will be associated to the previous [`BeginSubmenu`](Self::BeginSubmenu),
    /// or to the main menu if there wasn't any.
    EndSubmenu,
    /// A title or heading.
    Title {
        /// The text of the title.
        title: &'a CStr,
        /// Whether the entry is enabled.
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
        match kind {
            CLAP_CONTEXT_MENU_ITEM_ENTRY => {
                let data = data.cast::<clap_context_menu_entry>().as_ref()?;
                Some(Self::Entry {
                    enabled: data.is_enabled,
                    action_id: ClapId::from_raw(data.action_id)?,
                    label: cstr_from_nullable_ptr(data.label)?,
                })
            }
            CLAP_CONTEXT_MENU_ITEM_CHECK_ENTRY => {
                let data = data.cast::<clap_context_menu_check_entry>().as_ref()?;
                Some(Self::CheckEntry {
                    enabled: data.is_enabled,
                    action_id: ClapId::from_raw(data.action_id)?,
                    label: cstr_from_nullable_ptr(data.label)?,
                    checked: data.is_checked,
                })
            }
            CLAP_CONTEXT_MENU_ITEM_SEPARATOR => Some(Item::Separator),
            CLAP_CONTEXT_MENU_ITEM_BEGIN_SUBMENU => {
                let data = data.cast::<clap_context_menu_submenu>().as_ref()?;
                Some(Self::BeginSubmenu {
                    enabled: data.is_enabled,
                    label: cstr_from_nullable_ptr(data.label)?,
                })
            }
            CLAP_CONTEXT_MENU_ITEM_END_SUBMENU => Some(Item::EndSubmenu),
            CLAP_CONTEXT_MENU_ITEM_TITLE => {
                let data = data.cast::<clap_context_menu_item_title>().as_ref()?;
                Some(Self::Title {
                    enabled: data.is_enabled,
                    title: cstr_from_nullable_ptr(data.title)?,
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
