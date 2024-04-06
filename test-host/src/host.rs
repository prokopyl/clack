use clack_host::host::{AudioProcessorHandler, HostHandlers, MainThreadHandler, SharedHandler};

pub struct TestHostMainThread;
pub struct TestHostShared;
pub struct TestHostAudioProcessor;
pub struct TestHostImpl;

impl<'a> SharedHandler<'a> for TestHostShared {
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

impl<'a> AudioProcessorHandler<'a> for TestHostAudioProcessor {}

impl<'a> MainThreadHandler<'a> for TestHostMainThread {}

impl HostHandlers for TestHostImpl {
    type Shared<'a> = TestHostShared;
    type MainThread<'a> = TestHostMainThread;
    type AudioProcessor<'a> = TestHostAudioProcessor;
}
