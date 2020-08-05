use std::convert::TryInto;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::thread;
use std::time::Duration;

use hotbolt::{hotbolt_entry_main, hotbolt_entry_state, Server};

const COUNTER_DEFAULT: isize = 3;
static COUNTER: AtomicIsize = AtomicIsize::new(COUNTER_DEFAULT);

#[hotbolt_entry_main]
fn main(server: impl Server, state: &[u8]) {
	println!("In main entry point");
	let value = if state.is_empty() {
		println!("Using default value: {}", COUNTER_DEFAULT);
		COUNTER_DEFAULT
	} else {
		let value = isize::from_ne_bytes(state[0..8].try_into().expect("Deserialize state"));
		println!("Using provided value: {}", value);
		value
	};
	COUNTER.store(value, Ordering::Relaxed);

	loop {
		// fetch_sub returns the old value by default.
		let value = COUNTER.fetch_sub(1, Ordering::Relaxed);
		println!("Counter: {}", value);
		if value == 0 {
			server.reload();
		} else if value == -COUNTER_DEFAULT {
			server.restart();
		}
		thread::sleep(Duration::from_secs(1));
	}
}

#[hotbolt_entry_state]
fn state() -> Vec<u8> {
	let value = COUNTER.load(Ordering::Relaxed);
	value.to_ne_bytes().to_vec()
}
