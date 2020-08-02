# hotbolt
Turbo-charge your development with hot-reloading.

NOTE: This tool is proof of concept and does not work.

## Basic Usage
`hotbolt` works by running your application as a library.

Rename your `main.rs` to `lib.rs` and set your `Cargo.toml` file to build as a library:
```toml
[lib]
crate-type = ["cdylib"]
```

Put the hotbolt procedural macro in front of your `main`:
```rust
use std::thread;
use std::time::Duration;

use hotbolt::hotbolt_entry_main;

#[hotbolt_entry_main]
fn main() {
	let mut counter = 0;
	loop {
		println!("Counter: {}", counter);
		counter += 1;
		thread::sleep(Duration::from_secs(1));
	}
}
```

Finally, run `hotbolt-runner` with your library path as an argument:
```bash
cargo build
RUST_LOG=hotbolt_runner=debug hotbolt-runner
```

By default, this runs with the `debug` profile. To run with another one, specify it as such:
```bash
cargo build
RUST_LOG=hotbolt_runner=debug hotbolt-runner --profile release
```

### Automatically Rebuilding
You can use [`cargo-watch`](https://crates.io/crates/cargo-watch) for automatically rebuilding your library each time you make an edit for maximum efficiency!
```bash
cargo watch -x run
```

### Debug-only
For some projects, such as games, hot deployment is only intended during the development lifecycle. In this case, you want to build both a binary and a library.

The `hotbolt` macros by default generates some glue that you don't need in a binary. All of this glue can be disabled through the `hotbolt_erase` feature. Add the feature in your `Cargo.toml`:
```toml
[features]
hotbolt_erase = []
```

To tell `Cargo.toml` to continue building a library, you can keep your original `main.rs` and update your `Cargo.toml` as such:
```toml
[lib]
name = "hotbolt_runnable"
crate-type = ["cdylib"]
path = "src/main.rs"
```

Note, that Cargo emits a warning unless you use a different name for your library and your binary.

Cargo also emits a warning when both targets are using the same entry point. To get around this, you can remove the `path` field in `Cargo.toml` and add a `lib.rs` file that re-exports everything from `main.rs`. To prevent additional warnings, make it a conditional module:
```rust
#[cfg(not(feature = "hotbolt_erase"))]
mod main;
#[cfg(not(feature = "hotbolt_erase"))]
pub use main::*;
```

Then build your binary with the feature:
```bash
cargo build --release --features "hotbolt_erase"
```

## Examples
To run the examples, first build the root workspace, then build the examples workspace and finally run whichever example you want with `hotbolt_runner`:
```bash
cargo build
pushd examples
cargo build
popd
RUST_LOG=hotbolt_runner=debug cargo run "examples/counter"
```
