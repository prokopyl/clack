A crate offering low-level, safe Rust wrappers to create audio plugins using
the [CLAP](https://github.com/free-audio/clap) audio plugin API.

This library is made of lightweight, low-level wrappers built on top
of [`clap-sys`](https://crates.io/crates/clap-sys).

This is part of the [Clack](https://github.com/prokopyl/clack) project. See the [
`clack-host`](https://crates.io/crates/clack-host) crate if you need to make CLAP hosts, and the
[`clack-extensions`](https://crates.io/crates/clack-extensions) crate that contains wrappers for all the standard
CLAP extensions (for both plugins and hosts).

## Features

* **Safe**: The #1 goal of this project is to provide the full functionality of the CLAP plugin APIs through
  fully memory-safe and thread-safe wrappers.

  This allows for making safer, less crash-prone plugins, but also allows to fully trust Rust's
  type system when making large refactorings or performance optimizations, especially when dealing with CLAP's
  powerful, multithreaded model.
* **Low-level**: When safe designs permit it, Clack aims to be as close as possible to the underlying
  [CLAP](https://github.com/free-audio/clap) C API. Whenever it is safe to do so, it does not make any assumptions or
  interpretations about any operation, and simply passes data through to the user.

  Anything that's possible to do with the CLAP C API, should also be possible to do using only safe Rust and Clack.
* **Lightweight**: Considering the performance sensitivity of audio plugins, Clack aims to add as little
  overhead as possible to any operation (being a low-level library helps!). It reduces runtime checks and memory
  allocations to an absolute minimum, heavily leveraging zero-cost abstractions or user-provided buffers for example.
* **Extensible**: Following the intent of the underlying CLAP API, Clack allows third-party users and crates to use,
  add, or extend extensions, event types, factories, and more. As a matter of fact, all extensions in the
  `clack-extensions` crate are implemented using tools all Clack users have access to, and users may add or replace
  the provided implementations with their own!
* **Reasonably defensive**: While Clack performs very little extra sanity checks to keep its overhead as low as
  possible,
  it will report any erroneous behavior it stumbles upon to user code if possible, or just log it and fall back to
  the safest option otherwise (e.g. in case of panics originating from user code).
* **Fairly ergonomic**: Despite Clack being a low-level, fast API, it also tries to provide some additional ergonomic
  APIs
  when wrapping the C APIs, as long as they don't have any impact on performance or low-level capability. Examples
  include using Rust's `Option`, `Result`, allowing the use of iterators, implementing standard traits for e.g. I/O,
  `From` implementations for buffer types, etc.

## `clack-plugin` example

This example code implements a very simple gain plugin that amplifies every input by `2.0`. More involved
examples are available in the [examples](./plugin/examples) directory, and in the crate's documentation.

```rust
use clack_plugin::prelude::*;

pub struct MyGainPlugin;

impl Plugin for MyGainPlugin {
    type AudioProcessor<'a> = MyGainPluginAudioProcessor;

    type Shared<'a> = ();
    type MainThread<'a> = ();
}

impl DefaultPluginFactory for MyGainPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("org.rust-audio.clack.gain", "Clack Gain Example")
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        _shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(())
    }
}

pub struct MyGainPluginAudioProcessor;

impl<'a> PluginAudioProcessor<'a, (), ()> for MyGainPluginAudioProcessor {
    fn activate(_host: HostAudioProcessorHandle<'a>, _main_thread: &mut (), _shared: &'a (), _audio_config: PluginAudioConfiguration) -> Result<Self, PluginError> {
        Ok(Self)
    }

    fn process(&mut self, _process: Process, mut audio: Audio, _events: Events) -> Result<ProcessStatus, PluginError> {
        for mut port_pair in &mut audio {
            // For this example, we'll only care about 32-bit sample data.
            let Some(channel_pairs) = port_pair.channels()?.into_f32() else { continue; };

            for channel_pair in channel_pairs {
                match channel_pair {
                    ChannelPair::InputOnly(_) => {}
                    ChannelPair::OutputOnly(buf) => buf.fill(0.0),
                    ChannelPair::InputOutput(input, output) => {
                        for (input, output) in input.iter().zip(output) {
                            *output = input * 2.0
                        }
                    }
                    ChannelPair::InPlace(buf) => {
                        for sample in buf {
                            *sample *= 2.0
                        }
                    }
                }
            }
        }

        Ok(ProcessStatus::Continue)
    }
}

clack_export_entry!(SinglePluginEntry<MyGainPlugin>);
```

Compiling this code as a ["cdylib" library crate](https://doc.rust-lang.org/reference/linkage.html#r-link.cdylib) will
produce a CLAP-compliant dynamic library file in your project's `target` folder.

## License

This crate is distributed under the terms of both the [MIT license](LICENSE-MIT) and
the [Apache license, version 2.0](LICENSE-APACHE).
Contributions are accepted under the same terms.
