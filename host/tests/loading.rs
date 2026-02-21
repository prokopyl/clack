use clack_host::entry::PluginEntry;
use clack_host::factory::plugin::PluginFactory;

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support calling foreign function (dlopen)
pub fn it_works() {
    let library_path = format!(
        "{}/../target/debug/{}clack_plugin_gain{}",
        env!("CARGO_MANIFEST_DIR"),
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    // SAFETY: we made the plugin, if it's not UB-free then this is what this test is for :)
    let entry = unsafe { PluginEntry::load(library_path).unwrap() };

    let desc = entry
        .get_factory::<PluginFactory>()
        .unwrap()
        .plugin_descriptor(0)
        .unwrap();
    assert_eq!(desc.id().unwrap().to_bytes(), b"org.rust-audio.clack.gain");
}

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support calling foreign function (dlopen)
pub fn it_works_concurrently() {
    let entry_path = format!(
        "{}/../target/debug/{}clack_plugin_gain{}",
        env!("CARGO_MANIFEST_DIR"),
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );

    std::thread::scope(|s| {
        for _ in 0..300 {
            s.spawn(|| {
                // SAFETY: same as test above
                let entry = unsafe { PluginEntry::load(&entry_path).unwrap() };

                let desc = entry
                    .get_factory::<PluginFactory>()
                    .unwrap()
                    .plugin_descriptor(0)
                    .unwrap();
                assert_eq!(desc.id().unwrap().to_bytes(), b"org.rust-audio.clack.gain");
            });
        }
    })
}
