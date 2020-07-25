# hotbolt
Lightweight set of tools for hot-reloading in Rust.

## Usage
`hotbolt` works by transforming your application into a library.

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
RUST_LOG=hotbolt_runner=debug hotbolt-runner target/debug/hotbolt_runnable.dll
```

### Debug-only
For some projects, such as games, hot deployment is only intended during the development lifecycle. In this case, you want to build both a binary and a library.

To tell `Cargo.toml` to build both, you can keep your original `main.rs` and update your `Cargo.toml` as so:
```toml
[lib]
name = "hotbolt_runnable"
crate-type = ["cdylib"]
path = "src/main.rs"
```

Note, that you need to use a different name on your library than your binary. Additionally, `cargo` emits a warning that both targets are using the same entry point. To get around this, you can remove the `path` field in `Cargo.toml` and add a `lib.rs` file that re-exports everything from `main.rs`:
```rust
mod main;
pub use main::*;
```

The `hotbolt` macros by default generates some glue that you don't need in a binary. All of this "glue" can be disabled through the `hotbolt_erase` feature. Add the feature in your `Cargo.toml`:
```toml
[features]
hotbolt_erase = []
```

Then build your binary with the feature:
```bash
cargo build --release --features "hotbolt_erase"
```

The library target is still built with these configurations, but `hotbolt-runner` is incapable of utilitizing it. `cargo` also generates a warning on the `main` function being unused from the library. If you use a separate `lib.rs` file, you can make loading the main module conditional:
```rust
#[cfg(not(feature = "hotbolt_erase"))]
mod main;
#[cfg(not(feature = "hotbolt_erase"))]
pub use main::*;
```

## Examples
To run the examples, first build the root workspace, then build the examples workspace and finally run whichever example you want with `hotbolt_runner`:
```bash
cargo build
pushd examples
cargo build
popd
RUST_LOG=hotbolt_runner=debug cargo run examples/target/debug/hotbolt_runnable.dll
```
