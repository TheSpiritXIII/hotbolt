use std::{
	ffi::c_void,
	io::{Read, Write},
	os::raw::c_char,
};

mod common;
mod convert;

// TODO: Should these go to prelude?
pub use common::*;

pub mod prelude {
	pub use crate::convert::*;
}

// # NEW ENTRIES #
pub static ENTRY_CLIENT_NEW: &str = "hotbolt_entry_client_new";
pub static ENTRY_CLIENT_DROP: &str = "hotbolt_entry_client_get";
/*
TODO: What are the security implications of this?
Maybe we can have the client potentially be another (unreloadable) library?
Would anybody want these for beter logic separation/performance/ease?
*/
pub static ENTRY_CLIENT_VERSION: &str = "hotbolt_entry_client_version";
pub static ENTRY_CLIENT_COMPATIBLE: &str = "hotbolt_entry_client_compatible";

pub static ENTRY_STATE_NEW: &str = "hotbolt_entry_state_new";
pub static ENTRY_STATE_DROP: &str = "hotbolt_entry_state_drop";
pub static ENTRY_STATE_GET: &str = "hotbolt_entry_state_get";

pub static ENTRY_RUN: &str = "hotbolt_entry_run";

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

		unsafe extern "C" fn server_ffi_reload_with<T: Server>(
			arg: *const c_void,
			state: SizedCharArray,
		) {
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

/// Serializes and deserializes the application state.
pub trait Serializer<T> {
	// Writes the given value into the writer.
	fn serialize<W: Write>(writer: &W, value: &T) -> Result<(), ()>;

	// Reads the given value as T from the reader.
	fn deserialize<R: Read>(reader: R) -> Result<T, ()>;
}

pub trait Compatibility {
	// Returns a static string indicating the version number of the application.
	fn version() -> &'static [u8];

	// Returns true if the given other version is compatible with this one.
	fn is_compatible(other: &[u8]) -> bool {
		Self::version() == other
	}
}

/// Represents the application and its state.
pub trait Client {
	type T;
	type Serializer: Serializer<Self::T>;
	type Compatibility: Compatibility;

	/// Creates the client. Only called on initialization or if the client is incompatible with the last run client.
	fn new() -> Self;

	/// The main entry point for the application.
	fn run(&mut self, server: impl Server, state: &Box<Self::T>);

	/// Returns the current application state.
	fn state(&self) -> Option<Self::T>;
}

/// A low level version of `Client`.
pub trait FfiClient {
	fn client_new() -> *const c_void;
	fn client_drop(client_ptr: *mut c_void);

	fn client_version() -> SizedCharArray;
	fn client_compatible(other: SizedCharArray) -> bool;

	fn state_new(state_serialized: SizedCharArray) -> *const c_void;
	fn state_drop(state_ptr: *mut c_void);
	fn state_serialized(state_ptr: *const c_void) -> SizedCharArray;

	fn run(&mut self, server: FfiServer, state_ptr: *const c_void);
}

impl<T: Client> FfiClient for T {
	fn client_new() -> *const c_void {
		let client = Box::new(T::new());
		Box::into_raw(client) as *const c_void
	}

	fn client_drop(client_ptr: *mut c_void) {
		unsafe { Box::from_raw(client_ptr as *mut T) };
	}

	fn client_version() -> SizedCharArray {
		let version = T::Compatibility::version();
		let char_array = SizedCharArray {
			array: version.as_ptr() as *const c_char,
			len: version.len(),
		};
		std::mem::forget(version);
		char_array
	}

	fn client_compatible(other: SizedCharArray) -> bool {
		T::Compatibility::is_compatible(other.as_u8_slice());
		todo!();
	}

	fn state_new(state_serialized: SizedCharArray) -> *const c_void {
		if let Some(state) = T::Serializer::deserialize(state_serialized.as_u8_slice()).ok() {
			let state_ptr = Box::new(state);
			Box::into_raw(state_ptr) as *const c_void
		} else {
			std::ptr::null()
		}
	}

	fn state_drop(state_ptr: *mut c_void) {
		unsafe { Box::from_raw(state_ptr as *mut T::T) };
	}

	fn state_serialized(state_ptr: *const c_void) -> SizedCharArray {
		let state: Box<T::T> = unsafe { Box::from_raw(state_ptr as *mut T::T) };
		let vec: Vec<u8> = Vec::new();
		let value = if T::Serializer::serialize(&vec, &state).is_ok() {
			let char_array = SizedCharArray {
				array: vec.as_ptr() as *const c_char,
				len: vec.len(),
			};
			std::mem::forget(vec);
			char_array
		} else {
			SizedCharArray::empty()
		};
		Box::leak(state);
		value
	}

	fn run(&mut self, server: FfiServer, state_ptr: *const c_void) {
		let state: Box<T::T> = unsafe { Box::from_raw(state_ptr as *mut T::T) };
		Client::run(self, server, &state);
		Box::leak(state);
	}
}
