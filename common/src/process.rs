use clap_sys::process::*;

#[repr(i32)]
pub enum ProcessStatus {
    Continue = CLAP_PROCESS_CONTINUE,
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    Sleep = CLAP_PROCESS_SLEEP,
}
