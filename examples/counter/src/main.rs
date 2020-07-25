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
