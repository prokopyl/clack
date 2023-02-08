use clap_sys::events::clap_event_transport;
use clap_sys::process::*;

#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessStatus {
    Continue = CLAP_PROCESS_CONTINUE,
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    Sleep = CLAP_PROCESS_SLEEP,
    Tail = CLAP_PROCESS_TAIL,
}

impl ProcessStatus {
    pub fn from_raw(raw: clap_process_status) -> Option<Result<Self, ()>> {
        use ProcessStatus::*;

        match raw {
            CLAP_PROCESS_CONTINUE => Some(Ok(Continue)),
            CLAP_PROCESS_CONTINUE_IF_NOT_QUIET => Some(Ok(ContinueIfNotQuiet)),
            CLAP_PROCESS_SLEEP => Some(Ok(Sleep)),
            CLAP_PROCESS_TAIL => Some(Ok(Tail)),
            CLAP_PROCESS_ERROR => Some(Err(())),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct Transport {
    inner: clap_event_transport,
}
