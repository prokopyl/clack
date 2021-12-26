# rust-clap

A set of crates offering safe Rust wrappers to create audio plugins and hosts using the [CLAP](https://github.com/free-audio/clap) audio API.

This library is an exploratory attempt to make safe bindings to CLAP, built on top of [`clap-sys`](https://github.com/glowcoil/clap-sys).

It should be considered **highly experimental** and subject to change, and is far from being ready for production use.

This library is also very incomplete. At the moment, there is barely enough for an example Gain plugin to run.

## License
`rust-clap` is distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache license, version 2.0](LICENSE-APACHE).
Contributions are accepted under the same terms.