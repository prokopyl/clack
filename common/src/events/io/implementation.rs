use crate::events::io::TryPushError;
use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, UnknownEvent};
use crate::utils::handle_panic;
use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};

#[allow(clippy::len_without_is_empty)] // This is not necessary, the trait is intended for FFI
pub trait InputEventBuffer<'a>: Sized {
    fn len(&self) -> u32;
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>>;
}

pub trait OutputEventBuffer<'a>: Sized {
    fn try_push(&mut self, event: &UnknownEvent<'a>) -> Result<(), TryPushError>;
}

pub(crate) const fn raw_input_events<'a, I: InputEventBuffer<'a>>(buffer: &I) -> clap_input_events {
    clap_input_events {
        ctx: buffer as *const I as *mut I as *mut _,
        size: Some(size::<I>),
        get: Some(get::<I>),
    }
}

pub(crate) fn raw_output_events<'a, I: OutputEventBuffer<'a>>(
    buffer: &mut I,
) -> clap_output_events {
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
unsafe extern "C" fn size<'a, I: InputEventBuffer<'a>>(list: *const clap_input_events) -> u32 {
    handle_panic(|| I::len(&*((*list).ctx as *const _))).unwrap_or(0)
}

unsafe extern "C" fn get<'a, I: InputEventBuffer<'a>>(
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

unsafe extern "C" fn try_push<'a, O: OutputEventBuffer<'a>>(
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

impl<'a, T: Event<'a>> InputEventBuffer<'a> for T {
    #[inline]
    fn len(&self) -> u32 {
        1
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        match index {
            0 => Some(self.as_unknown()),
            _ => None,
        }
    }
}

impl<'a, T: Event<'a>, const N: usize> InputEventBuffer<'a> for [T; N] {
    #[inline]
    fn len(&self) -> u32 {
        N.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        self.as_slice().get(index as usize).map(|e| e.as_unknown())
    }
}

impl<'a, T: Event<'a>> InputEventBuffer<'a> for &[T] {
    #[inline]
    fn len(&self) -> u32 {
        let len = <[T]>::len(self);
        len.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        <[T]>::get(self, index as usize).map(|e| e.as_unknown())
    }
}

impl<'a> InputEventBuffer<'a> for &UnknownEvent<'a> {
    #[inline]
    fn len(&self) -> u32 {
        1
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        match index {
            0 => Some(self),
            _ => None,
        }
    }
}

impl<'a, const N: usize> InputEventBuffer<'a> for [&UnknownEvent<'a>; N] {
    #[inline]
    fn len(&self) -> u32 {
        N.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        self.as_slice().get(index as usize).copied()
    }
}

impl<'a> InputEventBuffer<'a> for &[&UnknownEvent<'a>] {
    #[inline]
    fn len(&self) -> u32 {
        let len = <[&UnknownEvent<'a>]>::len(self);
        len.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        <[&UnknownEvent<'a>]>::get(self, index as usize).copied()
    }
}

impl<'a, T: InputEventBuffer<'a>> InputEventBuffer<'a> for Option<T> {
    #[inline]
    fn len(&self) -> u32 {
        match self {
            Some(b) => b.len(),
            None => 0,
        }
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        match self {
            None => None,
            Some(b) => b.get(index),
        }
    }
}

impl<'a, T: Event<'a, EventSpace = CoreEventSpace<'a>> + Clone> OutputEventBuffer<'a>
    for Option<T>
{
    fn try_push(&mut self, event: &UnknownEvent<'a>) -> Result<(), TryPushError> {
        if self.is_some() {
            return Err(TryPushError);
        };

        if let Some(event) = event.as_event::<T>() {
            *self = Some(event.clone());
            Ok(())
        } else {
            Err(TryPushError)
        }
    }
}
