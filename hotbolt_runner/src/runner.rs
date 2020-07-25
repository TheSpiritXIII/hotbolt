use std::sync::mpsc::Receiver;

use libloading as lib;
use log::{debug, error, info};

pub enum Event {
	/// Confirmation that it is probably safe to start such as when a file is recently created.
	Start,
	/// Clear the current state and reload the library.
	Restart,
	/// Keep the current state and reload the library.
	Reload,
	/// Sets the current state.
	SetState(Box<[u8]>),
}

pub fn run(filepath: &str, reciever: Receiver<Event>) {
	let mut _state: Option<Box<[u8]>> = None;
	'lib_load: loop {
		let load_error;

		if let Ok(lib) = lib::Library::new(filepath) {
			info!("Successfully loaded library");
			// TODO: Call init symbol with state.

			let entry_point = unsafe {
				let func: Result<lib::Symbol<unsafe extern "C" fn() -> ()>, _> =
					lib.get(hotbolt_ffi::ENTRY_MAIN.as_bytes());
				func
			};
			if let Ok(symbol) = entry_point {
				debug!("Successfully loaded entry point");
				loop {
					if let Ok(event) = reciever.try_recv() {
						match event {
							Event::Start => {
								// Ignore this.
							}
							Event::Restart => {
								info!("Restarting application");
								_state = None;
								continue 'lib_load;
							}
							Event::Reload => {
								info!("Reloading library");
								// TODO: Update state.
								_state = None;
								continue 'lib_load;
							}
							Event::SetState(update) => {
								info!("Updating state");
								_state = Some(update);
							}
						}
					}
					unsafe {
						symbol();
					}
				}
			} else {
				error!("Error loading entry point");
				load_error = true;
			}
		} else {
			error!("Unable to load library");
			load_error = true;
		}

		// Must now loop until we get confirmation to restart.
		if load_error {
			error!("Due to previous failure, waiting for new file...");
			loop {
				if let Ok(event) = reciever.recv() {
					if let Event::Start = event {
						continue 'lib_load;
					}
				}
			}
		}
	}
}
