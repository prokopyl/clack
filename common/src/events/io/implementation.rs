use crate::events::io::TryPushError;
use crate::events::{Event, UnknownEvent};
use crate::utils::handle_panic;
use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};

#[allow(clippy::len_without_is_empty)] // This is not necessary, the trait is intended for FFI
pub trait InputEventBuffer: Sized {
    fn len(&self) -> u32;
    fn get(&self, index: u32) -> Option<&UnknownEvent>;
}

pub trait OutputEventBuffer: Sized {
    fn try_push(&mut self, event: &UnknownEvent<'static>) -> Result<(), TryPushError>;
}

pub(crate) const fn raw_input_events<I: InputEventBuffer>(buffer: &I) -> clap_input_events {
    clap_input_events {
        ctx: buffer as *const I as *mut I as *mut _,
        size: Some(size::<I>),
        get: Some(get::<I>),
    }
}

pub(crate) fn raw_output_events<I: OutputEventBuffer>(buffer: &mut I) -> clap_output_events {
    clap_output_events {
        ctx: buffer as *mut _ as *mut _,
        try_push: Some(try_push::<I>),
    }
}

pub(crate) const fn void_output_events() -> clap_output_events {
    clap_output_events {
        ctx: core::ptr::null_mut(),
        try_push: Some(void_push),
    }
}
unsafe extern "C" fn size<I: InputEventBuffer>(list: *const clap_input_events) -> u32 {
    handle_panic(|| I::len(&*((*list).ctx as *const _))).unwrap_or(0)
}

unsafe extern "C" fn get<I: InputEventBuffer>(
    list: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    handle_panic(|| {
        I::get(&*((*list).ctx as *const _), index)
            .map(|e| e.as_raw() as *const _)
            .unwrap_or_else(core::ptr::null)
    })
    .unwrap_or(core::ptr::null())
}

unsafe extern "C" fn try_push<O: OutputEventBuffer>(
    list: *const clap_output_events,
    event: *const clap_event_header,
) -> bool {
    handle_panic(|| {
        O::try_push(
            &mut *((*list).ctx as *const _ as *mut O),
            UnknownEvent::from_raw(event),
        )
        .is_ok()
    })
    .unwrap_or(false)
}

unsafe extern "C" fn void_push(
    _list: *const clap_output_events,
    _event: *const clap_event_header,
) -> bool {
    true
}

impl<T: Event<'static>, const N: usize> InputEventBuffer for [T; N] {
    #[inline]
    fn len(&self) -> u32 {
        N.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        self.as_slice().get(index as usize).map(|e| e.as_unknown())
    }
}

impl<'a, T: Event<'a>> InputEventBuffer for &'a [T] {
    #[inline]
    fn len(&self) -> u32 {
        let len = <[T]>::len(self);
        len.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        <[T]>::get(self, index as usize).map(|e| e.as_unknown())
    }
}

impl<'a, const N: usize> InputEventBuffer for [&UnknownEvent<'a>; N] {
    #[inline]
    fn len(&self) -> u32 {
        N.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        self.as_slice().get(index as usize).copied()
    }
}

impl<'a> InputEventBuffer for &[&UnknownEvent<'a>] {
    #[inline]
    fn len(&self) -> u32 {
        let len = <[&UnknownEvent<'a>]>::len(self);
        len.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        <[&UnknownEvent<'a>]>::get(self, index as usize).copied()
    }
}
