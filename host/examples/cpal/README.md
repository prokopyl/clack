# clack-host-cpal

An example of a functional CLAP host based on the `clack-host` crate,
using [CPAL](https://github.com/RustAudio/cpal) for audio output.

This small(-ish) host will load and instantiate a given plugin, show its UI in a window,
feed it with MIDI input and output it to the system's default device using
[CPAL](https://github.com/RustAudio/cpal).

### Limitations

Due to CPAL not being able to open a stream in duplex-mode (handling both input and
output at the same time), this host only connects to one single audio output and doesn't
handle any input.

This means audio effects plugins that process an incoming signal, while technically functional
in this host, will only receive silence as an input. In practice, synthesizers and other
audio-generating plugins are better suited to test this example with.

## Features

This is just an example host, don't expect too much in terms of features. :)

* **Plugin Discovery**: Given a plugin ID, will scan the bundles in all the standard CLAP paths
  on the filesystem to try and find a matching plugin. Alternatively, a specific CLAP bundle path
  can be provided.
* **Cross-platform**: Can work on Windows, macOS and Linux, including opening GUIs, reading MIDI
  and outputting audio.
* **GUI suppport**: Can open GUIs using each OS's default GUI API, either in floating or embedded
  window modes, depending on what the plugin supports.
* **MIDI input support**: Can read MIDI events from an input device, and forward them to the plugin.
* **Mono or Stereo output**, based on the plugin's preferences: will query the plugin's audio port
  information to try and best match with what the system can offer. Failing that, will automatically
  downmix stereo plugins to a mono output if stereo isn't available, or the other way around.

## Usage

```text
A simple CLI host to load and run a single CLAP plugin.

At least one of the `--plugin-id` (`-p`) or the `--bundle-path` (`-b`) parameters
*must* be used to specify which plugin to load.

Usage: clack-host-cpal [OPTIONS]

Options:
  -b, --bundle-path <BUNDLE_PATH>
          Loads the plugin found in the CLAP bundle at the given path.

          If the bundle contains multiple plugins, this should be used in
          conjunction with the `--plugin-id` (`-p`) parameter to specify
          which one to load.

  -p, --plugin-id <PLUGIN_ID>
          Loads the CLAP plugin with the given unique ID.

          This will start to scan the filesystem in the standard CLAP paths,
          and load all CLAP bundles found in those paths to search for the plugin
          matchingthe given ID.

          If multiple plugins matching the given ID were found on the filesystem,
          this should be used in conjunction with the `--bundle-path` (`-b`)
          parameter to specify which file to load the plugin from.

  -h, --help
          Print help (see a summary with '-h')
```

## Dependencies

Although the use of the `clack` crates are the main focus, this example also relies on the
following dependencies:

* [CPAL](https://crates.io/crates/cpal), for audio output.
* [`clap`](https://crates.io/crates/clap) (not this one, the other one), to handle CLI arguments.
* [Crossbeam's MPSC channel](https://crates.io/crates/crossbeam-channel), for all the plugin's threads to communicate
  with the main thread.
* [`dirs-rs`](https://crates.io/crates/dirs), to locate standard system directories, and deduce where CLAP bundles
  are stored for plugin discovery.
* [`midir`](https://crates.io/crates/midir) to connect to a MIDI input device, and
  [`wmidi`](https://crates.io/crates/wmidi) to decode them to CLAP note events.
* [`rtrb`](https://crates.io/crates/rtrb) as a SPSC ringbuffer-based channel to send MIDI events from `midir`'s thread
  to CPAL's audio thread.
* [`walkdir`](https://crates.io/crates/walkdir) and [`rayon`](https://crates.io/crates/rayon), for multi-thread
  plugin discovery.
* [`winit`](https://crates.io/crates/winit), to create a window for plugin GUIs to embed into, and to drive the UI
  event loop.
