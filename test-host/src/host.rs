use clack_host::host::{AudioProcessorHoster, MainThreadHoster, PluginHoster, SharedHoster};

pub struct TestHostMainThread;
pub struct TestHostShared;
pub struct TestHostAudioProcessor;
pub struct TestHostImpl;

impl<'a> SharedHoster<'a> for TestHostShared {
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

impl AudioProcessorHoster for TestHostAudioProcessor {}

impl<'a> MainThreadHoster<'a> for TestHostMainThread {}

impl<'a> PluginHoster<'a> for TestHostImpl {
    type AudioProcessor = TestHostAudioProcessor;
    type Shared = TestHostShared;
    type MainThread = TestHostMainThread;
}
