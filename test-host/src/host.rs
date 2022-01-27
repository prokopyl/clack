use clack_host::host::{AudioProcessorHoster, PluginHoster, SharedHoster};

pub struct TestHostMainThread;
pub struct TestHostShared;
pub struct TestHostAudioProcessor;

impl SharedHoster for TestHostShared {
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

impl<'a> PluginHoster<'a> for TestHostMainThread {
    type AudioProcessor = TestHostAudioProcessor;
    type Shared = TestHostShared;
}
