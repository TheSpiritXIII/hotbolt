use std::{ffi::c_void, os::raw::c_char};

pub static ENTRY_VERSION: &str = "hotbolt_entry_version";
pub static ENTRY_INIT: &str = "hotbolt_entry_init";
pub static ENTRY_MAIN: &str = "hotbolt_entry_main";
pub static ENTRY_STATE: &str = "hotbolt_entry_state";

#[repr(C)]
pub struct SizedCharArray {
	pub array: *const c_char,
	pub len: usize,
}

impl SizedCharArray {
	pub fn from_slice(slice: &[u8]) -> Self {
		Self {
			array: slice.as_ptr() as *const c_char,
			len: slice.len(),
		}
	}

	pub fn empty() -> Self {
		SizedCharArray {
			array: std::ptr::null(),
			len: 0,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn as_slice(&self) -> &[c_char] {
		unsafe { std::slice::from_raw_parts(self.array, self.len) }
	}

	pub fn as_u8_slice(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.array as *const u8, self.len) }
	}
}

pub trait Server {
	/// Restarts the application by exiting and calling its initiliazer with an empty state.
	fn restart(&self);

	/// Saves the current state and restarts the application by exiting and calling its initiliazer.
	fn reload(&self);

	/// Restarts the application by exiting and calling its initiliazer with an given state.
	fn reload_with(&self, state: &[u8]);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiServer {
	pub server: *const c_void,
	pub restart: unsafe extern "C" fn(*const c_void),
	pub reload: unsafe extern "C" fn(*const c_void),
	pub reload_with: unsafe extern "C" fn(*const c_void, state: SizedCharArray),
}

impl FfiServer {
	pub fn from<T: Server>(server: &T) -> Self {
		unsafe extern "C" fn server_ffi_restart<T: Server>(arg: *const c_void) {
			let pointer: *const T = arg as *const T;
			let server: &dyn Server = &*pointer;
			server.restart();
		}

		unsafe extern "C" fn server_ffi_reload<T: Server>(arg: *const c_void) {
			let pointer: *const T = arg as *const T;
			let server: &dyn Server = &*pointer;
			server.reload();
		}

		unsafe extern "C" fn server_ffi_reload_with<T: Server>(arg: *const c_void, state: SizedCharArray) {
			let pointer: *const T = arg as *const T;
			let server: &dyn Server = &*pointer;
			server.reload_with(state.as_u8_slice());
		}

		Self {
			server: server as *const T as *const c_void,
			restart: server_ffi_restart::<T>,
			reload: server_ffi_reload::<T>,
			reload_with: server_ffi_reload_with::<T>,
		}
	}
}

impl Server for FfiServer {
	fn restart(&self) {
		unsafe { (self.restart)(self.server) }
	}
	fn reload(&self) {
		unsafe { (self.reload)(self.server) }
	}
	fn reload_with(&self, state: &[u8]) {
		unsafe { (self.reload_with)(self.server, SizedCharArray::from_slice(&state)) }
	}
}
