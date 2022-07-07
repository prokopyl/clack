use clack_host::host::{Host, HostAudioProcessor, HostMainThread, HostShared};

pub struct TestHostMainThread;
pub struct TestHostShared;
pub struct TestHostAudioProcessor;
pub struct TestHostImpl;

impl<'a> HostShared<'a> for TestHostShared {
    fn request_restart(&self) {
        unimplemented!()
    }

    fn request_process(&self) {
        unimplemented!()
    }

    fn request_callback(&self) {
        unimplemented!()
    }
}

impl<'a> HostAudioProcessor<'a> for TestHostAudioProcessor {}

impl<'a> HostMainThread<'a> for TestHostMainThread {}

impl<'a> Host<'a> for TestHostImpl {
    type AudioProcessor = TestHostAudioProcessor;
    type Shared = TestHostShared;
    type MainThread = TestHostMainThread;
}
