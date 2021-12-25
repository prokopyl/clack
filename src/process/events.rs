use clap_audio_core::events::list::EventList;
use clap_sys::process::clap_process;

pub struct ProcessEvents<'a> {
    pub input: &'a EventList<'a>,
    pub output: &'a mut EventList<'a>,
}

impl<'a> ProcessEvents<'a> {
    pub(crate) unsafe fn from_raw(process: *const clap_process) -> Self {
        Self {
            input: EventList::from_raw_mut((*process).in_events),
            output: EventList::from_raw_mut((*process).out_events),
        }
    }
}
