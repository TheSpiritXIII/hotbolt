# hotbolt
Turbo-charge your development with hot-reloading.

NOTE: This tool is proof of concept and constantly changing. DO NOT USE IT [yet]!

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

Finally, run `hotbolt-runner`:
```bash
cargo build
RUST_LOG=hotbolt_runner=debug hotbolt-runner
```

And viola! Each time your library changes, the runner will automatically detect and restart the application.

### CLI Options
By default, the runner uses the `debug` profile. To run with another profile, specify it using the `--profile` flag:
```bash
cargo build
RUST_LOG=hotbolt_runner=debug hotbolt-runner --profile release
```

As an alternative, you can specify the library directly using `--file`. The equivalent to the above on Windows would be:
```bash
cargo build
RUST_LOG=hotbolt_runner=debug hotbolt-runner --file target/debug/app.dll
```

The hotbolt runner supports `--help` for additional runner features and usage tips:
```bash
cargo build
RUST_LOG=hotbolt_runner=debug hotbolt-runner --help
```

### Automatically Rebuilding
You can use [`cargo-watch`](https://crates.io/crates/cargo-watch) for automatically rebuilding your library each time you make an edit for maximum efficiency:
```bash
cargo watch -x build
```

### Debug-only Lifecycle
For some projects, such as games, hot deployment is only intended during the development lifecycle. You can build both a binary and a library for the release and debug builds respectically.

The `hotbolt` macros can be effectively disabled through the `hotbolt_erase` feature. Add the `hotbolt_erase` feature in your `Cargo.toml` and the macro becomes a no-op:
```toml
[features]
hotbolt_erase = []
```

To tell `Cargo.toml` to build a library, you can keep your original `main.rs` and update your `Cargo.toml`:
```toml
[lib]
# Need to populate name or else Cargo emits a warning.
name = "hotbolt_runnable"
crate-type = ["cdylib"]
path = "src/main.rs"
```

Cargo emits a warning when both targets (binary and library) are using the same entry point. As an alternative, you can omit the `path` field in `Cargo.toml` and add a `lib.rs` file that re-exports everything from `main.rs`. To prevent additional warnings, make it a conditional module:
```rust
#[cfg(not(feature = "hotbolt_erase"))]
mod main;
#[cfg(not(feature = "hotbolt_erase"))]
pub use main::*;
```

Finally, build your binary:
```bash
cargo build --release --features "hotbolt_erase"
```

## Manually Restarting
It is useful to bind restarting to a keyboard shortcut or another event within your application. Your application can communicate with the runner application through the `Server` object which can be added an argument to your entry point:
```rust
use std::thread;
use std::time::Duration;

use hotbolt::{hotbolt_entry_main, Server};

#[hotbolt_entry_main]
fn main(server: impl Server<()>) {
	for i in 0..3 {
		println!("Counter: {}: ", i);
		thread::sleep(Duration::from_secs(1));
	}
	server.restart()
}
```

## Reloading State
hotbolt is capable of storing and reloading state between each each refresh. The only caveat is that hotbolt does not include any serialization mechanisms by default.

Add a second variable that takes in a slice (must be in that position!). When the application starts up, the slice is empty. Finally, to tell hotbolt how to serialize, implement the `#[hotbolt_entry_state]` macro that returns a `Vec<u8>`:
```rust
use std::thread;
use std::time::Duration;

use hotbolt::{hotbolt_entry_main, Server};

const COUNTER_DEFAULT: isize = 0;
static COUNTER: AtomicIsize = AtomicIsize::new(COUNTER_DEFAULT);

#[hotbolt_entry_main]
fn main(server: impl Server<()>, state: &[u8]) {
	println!("In main entry point");
	let value = if state.is_empty() {
		COUNTER_DEFAULT
	} else {
		// We subtract 1 because fetch_add returns the old value.
		isize::from_ne_bytes(state[0..8].try_into().expect("Deserialize state")) - 1
	};
	COUNTER.store(value, Ordering::Relaxed);

	loop {
		let i = COUNTER.fetch_add(1, Ordering::Relaxed);
		println!("Counter: {}: ", i);
		thread::sleep(Duration::from_secs(1));
	}
	server.restart()
}

#[hotbolt_entry_state]
fn state() -> Vec<u8> {
	let value = COUNTER.load(Ordering::Relaxed);
	value.to_ne_bytes().to_vec()
}
```

For convenience, an macro is provided that expects a `hotbolt::Client` trait implementation. This is a work in progress.

## Hard vs Soft Reloading
All reloading thus far has been hard reloading -- the entire application stops and restarts (but with the old state). Some applications, such as servers, have long running TCP connections or use some sort of protocol or API that they don't want to reconnect each time they restart the server. If the application hard reloads, you would need to reconnect each time. Meanwhile games or other GUI application display a window on the screen. Hard reloading those types of applications cause the window to close and reopen, flickering and pointlessly reinitiliazing the surface.

Soft reloading allows the application to partially shut down. By dividing your application into two parts, you can avoid reloading the code that stays mostly static and continue reloading only the parts of your code that contains logic. Effectively, the runner has 2 versions of your library loaded.

This is a work in progress.

Notice how I used the word "mostly static" earlier to describe the long-running part of your application state. Sometimes it does change and you want to detect that and perform a hard reload. hotbolt supports this by allowing you to specify a version string. Like serialization, hotbolt is minimal and doesn't define what "compatibility" for you (for example, SemVer), so that is also something you must implement (although various helpers exist).

This is a work in progress.

## Examples
To run the examples in this repository, first build the root workspace, then build the examples workspace and finally run whichever example you want with `hotbolt_runner`:
```bash
cargo build
pushd examples
cargo build
popd
RUST_LOG=hotbolt_runner=debug cargo run "examples/counter"
```

### Tests
There are tests available which invokes cargo to create, build and run a new project:
```bash
RUST_LOG=hotbolt_runner=debug cargo test -- --test-threads=1 --nocapture
```
