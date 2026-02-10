A small utility library to help you find your [CLAP](https://cleveraudio.org/) audio plugins!

```rust
use clack_finder::ClapFinder;

pub fn main() {
    for bundle_path in ClapFinder::from_standard_paths() {
        println!("Found possible CLAP bundle at: {bundle_path:?}");
        // Load the bundle using e.g. clack-host or libloading, etc.
    }
}
```
