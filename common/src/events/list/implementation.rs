use crate::events::UnknownEvent;
use crate::utils::handle_panic;
use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};

pub trait InputEventBuffer {
    fn len(&self) -> u32;
    fn get(&self, index: u32) -> Option<&UnknownEvent>;
}

pub trait OutputEventBuffer {
    fn push_back(&mut self, event: &UnknownEvent);
}

pub(crate) const fn raw_input_events<I: InputEventBuffer>(buffer: &I) -> clap_input_events {
    clap_input_events {
        ctx: buffer as *mut _ as *mut _,
        size: Some(size::<I>),
        get: Some(get::<I>),
    }
}

pub(crate) const fn raw_output_events<I: InputEventBuffer>(buffer: &I) -> clap_output_events {
    clap_output_events {
        ctx: buffer as *mut _ as *mut _,
        push_back,
    }
}

unsafe extern "C" fn size<I: InputEventBuffer>(list: *const clap_input_events) -> u32 {
    handle_panic(|| I::len(&*((*list).ctx as *const _))).unwrap_or(0)
}

unsafe extern "C" fn get<'a, I: InputEventBuffer>(
    list: *const clap_input_events,
    index: u32,
) -> *const clap_event_header {
    handle_panic(|| {
        I::get(&*((*list).ctx as *const _), index)
            .map(|e| e.as_raw() as *const _)
            .unwrap_or_else(::core::ptr::null)
    })
    .unwrap_or_else(::core::ptr::null)
}

unsafe extern "C" fn push_back<'a, O: OutputEventBuffer>(
    list: *const clap_output_events,
    event: *const clap_event_header,
) {
    let _ = handle_panic(|| {
        O::push_back(
            &mut *((*list).ctx as *const _ as *mut O),
            UnknownEvent::from_raw(&*event),
        )
    });
}
