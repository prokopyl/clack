use clack_common::events::{InputEvents, OutputEvents};
use clap_sys::process::clap_process;

pub struct ProcessEvents<'a> {
    pub input: &'a InputEvents<'a>,
    pub output: &'a mut OutputEvents<'a>,
}

impl<'a> ProcessEvents<'a> {
    pub(crate) unsafe fn from_raw(process: *const clap_process) -> Self {
        Self {
            input: InputEvents::from_raw(&*(*process).in_events),
            output: OutputEvents::from_raw_mut(&mut *((*process).out_events as *mut _)),
        }
    }
}
