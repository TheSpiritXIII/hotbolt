use log::{debug, info, error};
use libloading as lib;
use std::env;

fn main() {
	env_logger::init();

	if let Some(lib_path) = env::args().skip(1).take(1).next() {
		debug!("Library: {}", lib_path);
		if let Ok(lib) = lib::Library::new(lib_path) {
			info!("Successfully loaded library");
			let entry_point = unsafe {
				let func: Result<lib::Symbol<unsafe extern fn() -> ()>, _> = lib.get(b"main");
				func
			};
			if let Ok(symbol) = entry_point {
				debug!("Loaded entry point.");
				unsafe {
					symbol();
				}
			} else {
				error!("Error loading entry point.");
			}
		} else {
			error!("Unable to load library");
		}
	} else {
		error!("Must specify library path");
	}
}
