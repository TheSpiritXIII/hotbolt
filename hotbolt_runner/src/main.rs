pub mod cargo;
pub mod library;
pub mod runner;
pub mod watcher;

use std::sync::mpsc::channel;
use std::{
	env, mem,
	path::{Path, PathBuf},
	process,
};

use log::error;

fn library_path<P: AsRef<Path>>(input: P) -> Result<PathBuf, String> {
	let path = input.as_ref();
	if path.is_file() {
		Ok(path.to_owned())
	} else if path.is_dir() {
		cargo::cargo_target_lib_path(path, "debug")
	} else {
		Err(format!("Unknown filesystem type `{}`", path.display()))
	}
}

fn main() {
	env_logger::init();

	if let Some(filepath) = env::args().skip(1).take(1).next() {
		let lib_path = library_path(filepath).unwrap_or_else(|e| {
			error!("{}", e);
			error!("Unable to resolve file path. Aborting");
			process::exit(1);
		});

		let (sender, receiver) = channel();
		let watcher = match watcher::watch(&lib_path, sender) {
			Ok(watcher) => watcher,
			Err(e) => {
				error!("{}", e);
				process::exit(1);
			}
		};

		// We need watcher in scope for the entire application lifecycle.
		// We don't want it to deallocate and stop listening to events.
		mem::forget(watcher);

		runner::run(lib_path, receiver);
	} else {
		error!("Must specify library path");
		process::exit(1);
	}
}
