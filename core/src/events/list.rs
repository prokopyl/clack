use crate::events::list::noop::NoopEventList;
use crate::events::Event;
use clap_sys::events::{clap_event, clap_event_list};
use std::marker::PhantomData;

pub trait EventListImplementation<'a>: 'a {
    fn size(&self) -> usize;
    fn get(&self, index: usize) -> Option<&'a Event>;
    fn push_back(&mut self, event: &Event);
}

mod noop;

#[repr(C)]
pub struct EventList<'a> {
    list: clap_event_list,
    _lifetime: PhantomData<&'a clap_event_list>,
}

impl<'a> EventList<'a> {
    #[inline]
    pub fn from_raw(raw: &'a clap_event_list) -> &'a Self {
        // SAFETY: EventList has the same layout and is repr(C)
        unsafe { ::core::mem::transmute(raw) }
    }

    #[inline]
    pub fn to_raw(&self) -> *const clap_event_list {
        &self as *const _ as *const _
    }

    #[inline]
    pub fn no_op() -> Self {
        Self {
            _lifetime: PhantomData,
            list: clap_event_list {
                ctx: ::core::ptr::null_mut(),
                size: size::<NoopEventList>,
                get: get::<NoopEventList>,
                push_back: push_back::<NoopEventList>,
            },
        }
    }

    #[inline]
    pub fn from_implementation<E: EventListImplementation<'a>>(implementation: &'a mut E) -> Self {
        Self {
            _lifetime: PhantomData,
            list: clap_event_list {
                ctx: implementation as *mut _ as *mut _,
                size: size::<E>,
                get: get::<E>,
                push_back: push_back::<E>,
            },
        }
    }
}

unsafe extern "C" fn size<'a, E: EventListImplementation<'a>>(list: *const clap_event_list) -> u32 {
    E::size(&*(list.cast())) as u32
}

unsafe extern "C" fn get<'a, E: EventListImplementation<'a>>(
    list: *const clap_event_list,
    index: u32,
) -> *const clap_event {
    E::get(&*(list.cast()), index as usize)
        .map(Event::as_raw)
        .unwrap_or_else(::core::ptr::null)
}

unsafe extern "C" fn push_back<'a, E: EventListImplementation<'a>>(
    list: *const clap_event_list,
    event: *const clap_event,
) {
    E::push_back(
        &mut *(list as *const _ as *mut _),
        Event::from_raw_ref(&*event),
    )
}
