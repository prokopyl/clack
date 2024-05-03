use crate::events::io::TryPushError;
use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, UnknownEvent};
use crate::utils::handle_panic;
use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};

/// A trait for all types which can act as an ordered, indexed list of [`UnknownEvent`]s.
///
/// This is the backing implementation of an [`InputEvents`](crate::events::io::InputEvents), which
/// is how input events are shared from the CLAP host to the plugin.
///
/// Note that events are indexed using `u32` instead of the standard `usize`, to match the CLAP
/// specification.
#[allow(clippy::len_without_is_empty)] // This is not necessary, the trait is intended for FFI
pub trait InputEventBuffer: Sized {
    /// Returns the number of events in this list.
    fn len(&self) -> u32;
    /// Returns the event at the given `index`.
    ///
    /// If `index` is out of bounds, then this must return `None` instead.
    fn get(&self, index: u32) -> Option<&UnknownEvent>;
}

/// A trait for all types which can act as an ordered queue for outbound [`UnknownEvent`]s.
///
/// This is the backing implementation of an [`OutputEvents`](crate::events::io::OutputEvents), which
/// is how output events are shared from the CLAP plugin to the host.
///
/// Note that events are indexed using `u32` instead of the standard `usize`, to match the CLAP
/// specification.
pub trait OutputEventBuffer: Sized {
    /// Attempts to push a given event to the queue.
    ///
    /// # Errors
    ///
    /// This may return a [`TryPushError`] if the event couldn't be pushed for any reason (e.g. the
    /// underlying implementation ran out of buffer space).
    fn try_push(&mut self, event: &UnknownEvent) -> Result<(), TryPushError>;
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

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn size<I: InputEventBuffer>(list: *const clap_input_events) -> u32 {
    handle_panic(|| I::len(&*((*list).ctx as *const _))).unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
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

#[allow(clippy::missing_safety_doc)]
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

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn void_push(
    _list: *const clap_output_events,
    _event: *const clap_event_header,
) -> bool {
    true
}

impl<T: Event> InputEventBuffer for T {
    #[inline]
    fn len(&self) -> u32 {
        1
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        match index {
            0 => Some(self.as_unknown()),
            _ => None,
        }
    }
}

impl<T: InputEventBuffer, U: InputEventBuffer> InputEventBuffer for (&T, &U) {
    #[inline]
    fn len(&self) -> u32 {
        self.0.len() + self.1.len()
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        let first_len = self.0.len();
        if index < first_len {
            self.0.get(index)
        } else {
            // No underflow possible: we checked if index was >= first_len above
            self.1.get(index - first_len)
        }
    }
}

impl<T: Event, const N: usize> InputEventBuffer for [T; N] {
    #[inline]
    fn len(&self) -> u32 {
        N.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        self.as_slice().get(index as usize).map(|e| e.as_unknown())
    }
}

impl<T: Event> InputEventBuffer for &[T] {
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

impl<T: Event> InputEventBuffer for Vec<T> {
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

impl InputEventBuffer for &UnknownEvent {
    #[inline]
    fn len(&self) -> u32 {
        1
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        match index {
            0 => Some(self),
            _ => None,
        }
    }
}

impl<const N: usize> InputEventBuffer for [&UnknownEvent; N] {
    #[inline]
    fn len(&self) -> u32 {
        N.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        self.as_slice().get(index as usize).copied()
    }
}

impl InputEventBuffer for &[&UnknownEvent] {
    #[inline]
    fn len(&self) -> u32 {
        let len = <[&UnknownEvent]>::len(self);
        len.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        <[&UnknownEvent]>::get(self, index as usize).copied()
    }
}

impl InputEventBuffer for Vec<&UnknownEvent> {
    #[inline]
    fn len(&self) -> u32 {
        let len = <[&UnknownEvent]>::len(self);
        len.min((u32::MAX - 1) as usize) as u32
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        <[&UnknownEvent]>::get(self, index as usize).copied()
    }
}

impl<T: InputEventBuffer> InputEventBuffer for Option<T> {
    #[inline]
    fn len(&self) -> u32 {
        match self {
            Some(b) => b.len(),
            None => 0,
        }
    }

    #[inline]
    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        match self {
            None => None,
            Some(b) => b.get(index),
        }
    }
}

impl<'a, T: Event<EventSpace<'a> = CoreEventSpace<'a>> + Clone> OutputEventBuffer for Option<T> {
    fn try_push(&mut self, event: &UnknownEvent) -> Result<(), TryPushError> {
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

impl<'a, T: Event<EventSpace<'a> = CoreEventSpace<'a>> + Clone> OutputEventBuffer for Vec<T> {
    fn try_push(&mut self, event: &UnknownEvent) -> Result<(), TryPushError> {
        if let Some(event) = event.as_event::<T>() {
            self.push(event.clone());
            Ok(())
        } else {
            Err(TryPushError)
        }
    }
}
