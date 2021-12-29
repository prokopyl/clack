use clap_audio_host::bundle::PluginBundle;

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support calling foreign function (dlopen)
pub fn it_works() {
    let bundle_path = format!(
        "{}/../target/debug/{}gain{}",
        env!("CARGO_MANIFEST_DIR"),
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let bundle = PluginBundle::load(&bundle_path).unwrap();
    let entry = bundle.get_entry().unwrap();

    let desc = entry.plugin_descriptor(0).unwrap();
    assert_eq!(desc.id().unwrap(), "gain");
}
