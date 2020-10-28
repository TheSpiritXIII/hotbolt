use std::{path::Path, sync::mpsc::Sender};

use hotbolt_ffi::{FfiServer, Server, SizedCharArray};
use libloading::{Library, Symbol};

fn load_symbol<'a, T: 'a>(lib: &'a Library, name: &'static str) -> Result<Symbol<'a, T>, String> {
	unsafe {
		let func: Result<Symbol<T>, _> = lib.get(name.as_bytes());
		func
	}
	.map_err(|_err| format!("Error loading symbol `{}`", name))
}

pub struct HotboltLibMain<'a> {
	run: Symbol<'a, unsafe extern "C" fn(server: FfiServer, state: SizedCharArray) -> ()>,
}

impl<'a> HotboltLibMain<'a> {
	fn from(lib: &'a Library) -> Result<Self, String> {
		Ok(Self {
			run: load_symbol(lib, hotbolt_ffi::ENTRY_RUN)?,
		})
	}

	pub fn run<T: Server>(&self, server: &T, state: &[u8]) {
		unsafe {
			(self.run)(
				FfiServer::from::<T>(&server),
				SizedCharArray::from_slice(state),
			)
		}
	}
}

pub struct HotboltLibState<'a> {
	state: Symbol<'a, unsafe extern "C" fn() -> SizedCharArray>,
}

impl<'a> HotboltLibState<'a> {
	fn from(lib: &'a Library) -> Result<Self, String> {
		Ok(Self {
			state: load_symbol(lib, hotbolt_ffi::ENTRY_STATE_GET)?,
		})
	}

	pub fn state(&self) -> Box<[u8]> {
		// TODO: Avoid alloc?
		unsafe { (self.state)() }
			.as_u8_slice()
			.to_vec()
			.into_boxed_slice()
	}
}

pub struct HotboltLib {
	lib: Library,
	// server: &'a T,
}

impl HotboltLib {
	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
		Library::new(path.as_ref().as_os_str())
			.map(|lib| Self { lib })
			.map_err(|_err| "Error loading entry point".to_owned())
	}

	pub fn symbols(&self) -> Result<HotboltLibMain<'_>, String> {
		HotboltLibMain::from(&self.lib)
	}

	pub fn state(&self) -> Result<HotboltLibState<'_>, String> {
		HotboltLibState::from(&self.lib)
	}
}

#[derive(Debug)]
pub enum SenderEvent {
	Restart,
	Reload,
	ReloadWith(Box<[u8]>),
}

pub struct SenderServer {
	pub sender: Sender<SenderEvent>,
}

impl SenderServer {
	// TODO: Implement Display for Event?
	fn send(&self, event: SenderEvent, display: &'static str) {
		self.sender.send(event).unwrap_or_else(|_err| {
			panic!("hotbolt server `{}` message failed to send", display);
		});
	}
}

impl Server for SenderServer {
	fn restart(&self) {
		self.send(SenderEvent::Restart, "Restart");
	}
	fn reload(&self) {
		self.send(SenderEvent::Reload, "Reload");
	}
	fn reload_with(&self, state: &[u8]) {
		// TODO: avoid alloc?
		let box_slice = state.to_vec().into_boxed_slice();
		self.send(SenderEvent::ReloadWith(box_slice), "ReloadWith");
	}
}
