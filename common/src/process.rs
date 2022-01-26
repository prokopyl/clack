use clap_sys::events::clap_event_transport;
use clap_sys::process::*;

#[repr(i32)]
pub enum ProcessStatus {
    Continue = CLAP_PROCESS_CONTINUE,
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    Sleep = CLAP_PROCESS_SLEEP,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Transport {
    inner: clap_event_transport,
}
