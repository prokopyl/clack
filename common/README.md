A small crate containing various utilities and definitions for working with
the [CLAP](https://github.com/free-audio/clap) audio plugin API, for both
plugins and hosts.

This library is made of lightweight, low-level wrappers built on top
of [`clap-sys`](https://crates.io/crates/clap-sys).

This is part of the [Clack](https://github.com/prokopyl/clack) project. All modules of this crate are re-exported in the
[
`clack-plugin`](https://crates.io/crates/clack-plugin) and [
`clack-host`](https://crates.io/crates/clack-host) crates. Most users of those crates should not
have to use this crate directly.

However, this crate can also be used standalone in any project, without any dependency on any other Clack crates.

See this crate's documentation for a list of all the utility type it exposes.

## License

This crate is distributed under the terms of both the [MIT license](LICENSE-MIT) and
the [Apache license, version 2.0](LICENSE-APACHE).
Contributions are accepted under the same terms.
