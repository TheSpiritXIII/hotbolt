pub mod runner;
pub mod watcher;

use std::sync::mpsc::channel;
use std::{env, process, mem};

use log::error;

fn main() {
	env_logger::init();

	if let Some(filepath) = env::args().skip(1).take(1).next() {
		let (sender, receiver) = channel();
		let watcher = match watcher::watch(&filepath, sender) {
			Ok(watcher) => watcher,
			Err(e) => {
				error!("{}", e);
				process::exit(1);
			}
		};

		// We need watcher in scope for the entire application lifecycle.
		// We don't want it to deallocate and stop listening to events.
		mem::forget(watcher);

		runner::run(&filepath, receiver);
	} else {
		error!("Must specify library path");
		process::exit(1);
	}
}
