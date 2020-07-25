use std::thread;
use std::time::Duration;

#[no_mangle]
pub extern "C" fn main() {
	let mut counter = 0;
	loop {
		println!("Counter: {}", counter);
		counter += 1;
		thread::sleep(Duration::from_secs(1));
	}
}
