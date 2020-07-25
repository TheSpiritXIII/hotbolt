use libloading as lib;
use std::env;

fn main() {
	if let Some(lib_path) = env::args().skip(1).take(1).next() {
		println!("Library: {}", lib_path);
		if let Ok(lib) = lib::Library::new(lib_path) {
			println!("Successfully loaded library");
			let entry_point = unsafe {
				let func: Result<lib::Symbol<unsafe extern fn() -> ()>, _> = lib.get(b"main");
				func
			};
			if let Ok(symbol) = entry_point {
				println!("Loaded entry point.");
				unsafe {
					symbol();
				}
			} else {
				println!("Error loading entry point.");
			}
		} else {
			println!("Unable to load library");
		}
	} else {
		println!("Must specify library path");
	}
}
