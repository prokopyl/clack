# `bundle-plugins` helper command

This is a simple helper command that builds platform-appropriate CLAP bundle for every
`clack-plugin` example.

This helper is available in any directory in this repository/workspace and can be run
as a `cargo` subcommand, as the following:

```shell
cargo bundle-plugins
```

This will compile all plugins and create their respective `.clap` bundles in the `target/dist`
directory, located at the root of the workspace.
