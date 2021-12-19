use crate::process::audio::Audio;
use clap_sys::process::clap_process;

#[repr(C)]
pub struct Process {
    inner: clap_process,
}

impl Process {
    #[inline]
    pub(crate) fn from_raw(raw: &clap_process) -> &Process {
        // SAFETY: Process is repr(C) and is guaranteed to have the same memory representation
        unsafe { ::core::mem::transmute(raw) }
    }

    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.inner.frames_count
    }

    #[inline]
    pub fn steady_time(&self) -> u64 {
        self.inner.steady_time
    }

    #[inline]
    pub fn audio(&self) -> Audio {
        unsafe {
            Audio {
                frames_count: self.inner.frames_count,
                inputs: ::core::slice::from_raw_parts_mut(
                    self.inner.audio_inputs as *mut _,
                    self.inner.audio_inputs_count as usize,
                ),
                outputs: ::core::slice::from_raw_parts_mut(
                    self.inner.audio_outputs as *mut _,
                    self.inner.audio_outputs_count as usize,
                ),
            }
        }
    }
}

pub mod audio;
