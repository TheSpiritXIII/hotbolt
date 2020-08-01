use std::thread;
use std::time::Duration;

use hotbolt::{hotbolt_entry_main, Server};

#[hotbolt_entry_main]
fn main(server: impl Server) {
	let mut counter = 5;
	loop {
		println!("Counter: {}", counter);
		counter -= 1;
		thread::sleep(Duration::from_secs(1));
		if counter == 0 {
			server.restart();
		}
	}
}
