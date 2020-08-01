use std::{path::Path, sync::mpsc::Receiver};

use libloading::{Library, Symbol};
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

struct HotboltLibSymbols<'a> {
	run: Symbol<'a, unsafe extern "C" fn() -> ()>,
}

impl<'a> HotboltLibSymbols<'a> {
	fn from(lib: &'a Library) -> Result<Self, String> {
		Ok(Self {
			run: Self::load_symbol(lib, hotbolt_ffi::ENTRY_MAIN)?,
		})
	}

	fn load_symbol<T: 'a>(lib: &'a Library, name: &'static str) -> Result<Symbol<'a, T>, String> {
		unsafe {
			let func: Result<Symbol<T>, _> = lib.get(name.as_bytes());
			func
		}
		.map_err(|_err| format!("Error loading symbol `{}`", name))
	}

	fn run(&self) {
		unsafe { (self.run)() }
	}
}

struct HotboltLib {
	lib: Library,
}

impl HotboltLib {
	fn load<P: AsRef<Path>>(path: P) -> Result<HotboltLib, String> {
		Library::new(path.as_ref().as_os_str())
			.map(|lib| Self { lib })
			.map_err(|_err| "Error loading entry point".to_owned())
	}

	fn symbols<'a>(&'a self) -> Result<HotboltLibSymbols<'a>, String> {
		HotboltLibSymbols::from(&self.lib)
	}
}

pub fn run<P: AsRef<Path>>(path: P, reciever: Receiver<Event>) {
	let mut _state: Option<Box<[u8]>> = None;
	'lib_load: loop {
		let load_error;

		match HotboltLib::load(&path) {
			Ok(lib) => {
				info!("Successfully loaded library");
				
				match lib.symbols() {
					Ok(symbols) => {
						debug!("Successfully loaded entry point");
						// TODO: Call init symbol with state.

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
							symbols.run();
						}
					}
					Err(err) => {
						error!("{}", err);
						load_error = true;
					}
				}
			}
			Err(err) => {
				error!("{}", err);
				load_error = true;
			}
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
