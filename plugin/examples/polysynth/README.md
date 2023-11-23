# clack-plugin-polysynth
A small, polyphonic square wave synthesizer CLAP plugin, based on the `clack-plugin` crate.

### Features
This project is an example for the `clack-plugin` and `clack-extensions` crates, and shows
off the various parts of the Clack API by implementing the following features: 

* **General Clack plugin structure:** Usage and implementation of the `Plugin` trait, and of the
  `PluginMainThread`, `PluginAudioThread` and `PluginShared` sub-traits.
* **Audio output declaration and generation:** Using the `audio-ports` CLAP extension to declare
  audio ports, and accessing the various audio buffers in the `process` call.
* **Note input declaration and usage:** Using the `note-ports` CLAP extension to declare
  note ports, and sorting through the input events in the `process` call.
* **Parameter declaration, management and usage:** Using the `params` CLAP extension
  to declare parameters, format them for displaying to the user, and receiving updates
  from automation or the DAW's own UI.
* **State management:** Using the `state` CLAP extension to save the value of the
  parameter, so it can be restored later.
* **Aliasing:** This produces a *lot* of it. It's up to you to decide if this is a
  feature or not. 🙂

## Building and installing from source

To build this example from source, move (`cd`) to the directory containing
the Clack source code, and you can build the example using `cargo` like so:

```shell
cargo build -p clack-plugin-polysynth --release
```

This will create a `clack_plugin_polysynth` library file (suffix may vary depending on
your Operating System) in the `target/release` directory. 

You can then copy (or link) that file to your CLAP plugin directory, and renaming it
with a `.clap` extension (e.g. `clack_plugin_polysynth.clap`). This will enable it to
be picked up by your CLAP DAWs and hosts.

## Usage

This example plugin will show up as an "Clack PolySynth Example" instrument in your DAW
or host.

Upon loading, it will play a square wave for every note it receives.

It only has a single *Volume* parameter, which dictates the absolute level of
each voice.