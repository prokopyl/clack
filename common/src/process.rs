use clap_sys::events::clap_event_transport;
use clap_sys::process::*;

#[repr(i32)]
pub enum ProcessStatus {
    Continue = CLAP_PROCESS_CONTINUE as i32,
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET as i32,
    Sleep = CLAP_PROCESS_SLEEP as i32,
}

#[repr(C)]
pub struct Transport {
    inner: clap_event_transport,
}
