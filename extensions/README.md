A collection of all the standard [CLAP](https://github.com/free-audio/clap) extensions, for use
with the [clack-plugin](https://crates.io/crates/clack-plugin) or [clack-host](https://crates.io/crates/clack-host)
crates.

This is part of the [Clack](https://github.com/prokopyl/clack) project. See the [
`clack-plugin`](https://crates.io/crates/clack-plugin) crate if you need to make CLAP plugins, and the [
`clack-host`](https://crates.io/crates/clack-host) crate if you need to make CLAP hosts.

# Usage

You can pick and choose which extensions you want to be compiled into your project by enabling the corresponding
Cargo features (see the list below).

Then, you'll have to enable the `clack-host` feature if you want to use the extensions with the [
`clack-host`](https://crates.io/crates/clack-host) crate, or the `clack-host` feature if you want to use the extensions
with the [
`clack-plugin`](https://crates.io/crates/clack-plugin) crate.  
Both can also be selected if you need to work with both in your project.

Once enabled, the `clack_extensions` namespace will expose extension types
and associated utilities to use with either your host or plugin implementation.

See the crate documentation for a description and examples of each extension's APIs,
and the `clack-host` or `clack-plugin` crate documentations for more information on how to use CLAP extensions.

## Cargo features

### Clack integrations

* `clack-plugin`: Enables integration with the [`clack-plugin`](https://crates.io/crates/clack-plugin) crate.
* `clack-host`: Enables integration with the [`clack-host`](https://crates.io/crates/clack-host) crate.

### Extensions

When enabled, these features expose the associated extension wrapper types and utilities to be compiled into your
project.

* `all-extensions`: Enables all the extensions listed below.
* `ambisonic`: Exposes the [Ambisonic](https://github.com/free-audio/clap/blob/main/include/clap/ext/ambisonic.h)
  extension wrappers. This also enables the required `audio-ports` extension feature.
* `audio-ports`: Exposes the [Audio Ports](https://github.com/free-audio/clap/blob/main/include/clap/ext/audio-ports.h)
  extension wrappers.
* `audio-ports-activation`: Exposes
  the [Audio Ports Activation](https://github.com/free-audio/clap/blob/main/include/clap/ext/audio-ports-activation.h)
  extension wrappers.
* `audio-ports-config`: Exposes
  the [Audio Ports Config](https://github.com/free-audio/clap/blob/main/include/clap/ext/audio-ports-config.h) extension
  wrappers. This also enables the required `audio-ports`
  extension
  feature.
* `configurable-audio-ports`: Exposes
  the [Configurable Audio Ports](https://github.com/free-audio/clap/blob/main/include/clap/ext/configurable-audio-ports.h)
  extension wrappers. This also enables the required `audio-ports`
  extension feature.
* `context-menu`: Exposes
  the [Context Menu](https://github.com/free-audio/clap/blob/main/include/clap/ext/context-menu.h) extension wrappers.
* `clap-wrapper`: Exposes the Ambisonic extension wrappers.
* `event-registry`: Exposes
  the [Event Registry](https://github.com/free-audio/clap/blob/main/include/clap/ext/event-registry.h) extension
  wrappers.
* `gui`: Exposes the [GUI](https://github.com/free-audio/clap/blob/main/include/clap/ext/gui.h) extension wrappers.
* `latency`: Exposes the [Latency](https://github.com/free-audio/clap/blob/main/include/clap/ext/latency.h) extension
  wrappers.
* `log`: Exposes the [Log](https://github.com/free-audio/clap/blob/main/include/clap/ext/log.h) extension wrappers.
* `note-name`: Exposes the [Note Name](https://github.com/free-audio/clap/blob/main/include/clap/ext/note-name.h)
  extension wrappers.
* `note-ports`: Exposes the [Note Ports](https://github.com/free-audio/clap/blob/main/include/clap/ext/note-ports.h)
  extension wrappers.
* `params`: Exposes the [Params](https://github.com/free-audio/clap/blob/main/include/clap/ext/params.h)
  extension wrappers.
* `param-indication`: Exposes
  the [Param Indication](https://github.com/free-audio/clap/blob/main/include/clap/ext/param-indication.h) extension
  wrappers.
* `posix-fd`: Exposes the [POSIX FD](https://github.com/free-audio/clap/blob/main/include/clap/ext/posix-fd-support.h)
  extension wrappers.
* `preset-discovery`: Exposes
  the [Preset Load](https://github.com/free-audio/clap/blob/main/include/clap/ext/posix-fd-support.h) extension wrappers
  and the [Preset Discovery](https://github.com/free-audio/clap/blob/main/include/clap/factory/preset-discovery.h)
  factory wrappers.
* `remote-controls`: Exposes
  the [Remote Controls](https://github.com/free-audio/clap/blob/main/include/clap/ext/remote-controls.h) extension
  wrappers.
* `render`: Exposes the [Render](https://github.com/free-audio/clap/blob/main/include/clap/ext/render.h) extension
  wrappers.
* `state`: Exposes the [State](https://github.com/free-audio/clap/blob/main/include/clap/ext/state.h) extension
  wrappers.
* `state-context`: Exposes
  the [State Context](https://github.com/free-audio/clap/blob/main/include/clap/ext/state-context.h) extension wrappers.
  This also enables the required `state` extension
  feature.
* `surround`: Exposes the [Surround](https://github.com/free-audio/clap/blob/main/include/clap/ext/surround.h) extension
  wrappers. This also enables the required `audio-ports` extension feature.
* `tail`: Exposes the [Tail](https://github.com/free-audio/clap/blob/main/include/clap/ext/tail.h) extension wrappers.
* `thread-check`: Exposes
  the [Thread Check](https://github.com/free-audio/clap/blob/main/include/clap/ext/thread-check.h) extension wrappers.
* `thread-pool`: Exposes the [Thread Pool](https://github.com/free-audio/clap/blob/main/include/clap/ext/thread-pool.h)
  extension wrappers.
* `timer`: Exposes the [Timer Support](https://github.com/free-audio/clap/blob/main/include/clap/ext/timer-support.h)
  extension wrappers.
* `track-info`: Exposes the [Track Info](https://github.com/free-audio/clap/blob/main/include/clap/ext/track-info.h)
  extension wrappers. This also enables the required `audio-ports` extension
  feature.
* `voice-info`: Exposes the [Voice Info](https://github.com/free-audio/clap/blob/main/include/clap/ext/voice-info.h)
  extension wrappers.

### External integrations

These features enable integrations with third-party crates within the Rust ecosystem.

* `raw-window-handle_05`: Integrates the `Window` type from the GUI extension with the [
  `raw-window-handle`](https://crates.io/crates/raw-window-handle/0.5.2) crate,
  version 0.5.
* `raw-window-handle_06`: Integrates the `Window` type from the GUI extension with the [
  `raw-window-handle`](https://crates.io/crates/raw-window-handle) crate,
  version 0.6.
