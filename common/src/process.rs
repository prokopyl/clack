use clap_sys::events::clap_event_transport;
use clap_sys::process::*;

#[repr(i32)]
pub enum ProcessStatus {
    Continue = CLAP_PROCESS_CONTINUE as i32,
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET as i32,
    Sleep = CLAP_PROCESS_SLEEP as i32,
}

impl ProcessStatus {
    pub fn from_raw(raw: clap_process_status) -> Option<Result<Self, ()>> {
        use ProcessStatus::*;

        match raw as i32 {
            CLAP_PROCESS_CONTINUE => Some(Ok(Continue)),
            CLAP_PROCESS_CONTINUE_IF_NOT_QUIET => Some(Ok(ContinueIfNotQuiet)),
            CLAP_PROCESS_SLEEP => Some(Ok(Sleep)),
            CLAP_PROCESS_ERROR => Some(Err(())),
            _ => None,
        }
    }
}

#[repr(C)]
pub struct Transport {
    inner: clap_event_transport,
}
