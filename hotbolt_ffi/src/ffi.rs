use std::{ffi::c_void, marker::PhantomData};

use crate::{
	base::{App, AppVersion, Run, Server, ServerBase, ServerEnabled, State, StateConverter},
	common::{Deserializer, FfiArray, FfiArrayMut, Serializer},
	convert::UnsafeInto,
};

/// Server object sent over FFI. See [`Server`](Server).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiServer {
	pub server: *const c_void,
	pub restart_hard: unsafe extern "C" fn(server_ptr: *const c_void),
	pub restart_hard_with:
		unsafe extern "C" fn(server_ptr: *const c_void, state: FfiArrayMut<'static, u8>),
	pub restart_soft: unsafe extern "C" fn(server_ptr: *const c_void),
	pub restart_soft_with:
		unsafe extern "C" fn(server_ptr: *const c_void, state: FfiArrayMut<'static, u8>),
}

struct TypedFfiServer<T, U: Serializer<T>> {
	phantom_t: PhantomData<T>,
	phantom_u: PhantomData<U>,
	ffi_server: FfiServer,
}

impl<T, U: Serializer<T>> TypedFfiServer<T, U> {
	fn from(ffi_server: FfiServer) -> Self {
		TypedFfiServer {
			phantom_t: PhantomData,
			phantom_u: PhantomData,
			ffi_server,
		}
	}
}

impl<T, U: Serializer<T>> ServerEnabled for TypedFfiServer<T, U> {
	#[inline(always)]
	fn is_server_enabled() -> bool {
		true
	}
}

// TODO: Maybe expect/serialization errors can be more explicit
impl<T, S: Serializer<T>> ServerBase<T> for TypedFfiServer<T, S> {
	fn restart_hard(&self) {
		unsafe { (self.ffi_server.restart_hard)(self.ffi_server.server) }
	}

	fn restart_hard_with<U: AsRef<T>>(&self, state: U) {
		let serialized = S::serialize(state.as_ref()).expect("Serialized failed");
		unsafe { (self.ffi_server.restart_hard_with)(self.ffi_server.server, serialized) }
	}
}

impl<T, S: Serializer<T>> Server<T> for TypedFfiServer<T, S> {
	fn restart_soft(&self) {
		unsafe { (self.ffi_server.restart_soft)(self.ffi_server.server) }
	}

	fn restart_soft_with<U: AsRef<T>>(&self, state: U) {
		let serialized = S::serialize(state.as_ref()).expect("Serialized failed");
		unsafe { (self.ffi_server.restart_soft_with)(self.ffi_server.server, serialized) }
	}
}

/// The version of the hotbolt server that this library supports.
pub const SERVER_VERSION: u8 = 0;

/// The internal hotbolt server version this was written to support.
///
/// Signature: `() -> u8`
pub const ENTRY_SERVER_VERSION: &str = "hotbolt_entry_server_version";

/// Low level version of [`Run`](Run).
pub trait FfiRun {
	/// Runs the application. This is called in a loop.
	fn run(app_ptr: *mut c_void, server: FfiServer, state_ptr: *mut c_void);
}

impl<T: Run> FfiRun for T {
	fn run(app_ptr: *mut c_void, server: FfiServer, state_ptr: *mut c_void) {
		let state_typed_ptr = state_ptr as *mut <T::StateConverter as StateConverter>::State;
		let state: &mut <T::StateConverter as StateConverter>::State = unsafe { &mut *state_typed_ptr };

		let app_typed_ptr = app_ptr as *mut T::App;
		let app: &mut T::App = unsafe { &mut *app_typed_ptr };

		let server_typed = TypedFfiServer::<
			<T::StateConverter as StateConverter>::State,
			<T::StateConverter as StateConverter>::Serializer,
		>::from(server);
		T::run(app, server_typed, state);
	}
}

/// See [`FfiRun::run`](FfiRun::run).
pub const ENTRY_APP_RUN: &str = "hotbolt_entry_run";

/// Low level version of [`State`](State).
pub trait FfiState {
	/// Allocates and returns a new state from the potentially given serialized state.
	fn state_new(serialized: FfiArray<'static, u8>) -> *mut c_void;

	/// Drops the state. Skipped when possible. Do not use this for side-effects.
	fn state_drop(state_ptr: *mut c_void);

	/// Serializes the state as an array of bytes.
	fn state_serialized_new(state_ptr: *const c_void) -> FfiArrayMut<'static, u8>;

	/// Drops the given array of bytes. Skipped when possible. Do not use this for side-effects.
	fn state_serialized_drop(serialized: FfiArrayMut<'static, u8>);
}

impl<T: StateConverter> FfiState for T {
	fn state_new(state_serialized: FfiArray<u8>) -> *mut c_void {
		if !state_serialized.is_empty() {
			let slice = unsafe { state_serialized.as_slice() };

			if let Some(state) = T::Deserializer::deserialize(slice).ok() {
				let state_ptr = Box::new(state);
				Box::into_raw(state_ptr).cast()
			} else {
				Box::into_raw(Box::new(T::State::new())).cast()
			}
		} else {
			Box::into_raw(Box::new(T::State::new())).cast()
		}
	}

	fn state_drop(state_ptr: *mut c_void) {
		unsafe { Box::from_raw(state_ptr as *mut T::State) };
	}

	fn state_serialized_new(state_ptr: *const c_void) -> FfiArrayMut<'static, u8> {
		let state: &T::State = unsafe { &*state_ptr.cast() };
		T::Serializer::serialize(state).unwrap_or(FfiArrayMut::<u8>::empty())
	}

	fn state_serialized_drop(serialized: FfiArrayMut<'static, u8>) {
		let vec: Vec<u8> = unsafe { serialized.unsafe_into() };
		std::mem::drop(vec)
	}
}

/// See [`FfiState::state_new`](FfiState::state_new).
pub const ENTRY_STATE_NEW: &str = "hotbolt_entry_state_new";

/// See [`FfiState::state_drop`](FfiState::state_drop).
pub const ENTRY_STATE_DROP: &str = "hotbolt_entry_state_drop";

/// See [`FfiState::state_serialized_new`](FfiState::state_serialized_new).
pub const ENTRY_STATE_SERIALIZE_NEW: &str = "hotbolt_entry_state_serialize_new";

///  See [`FfiState::state_serialized_drop`](FfiState::state_serialized_drop).
pub const ENTRY_STATE_SERIALIZE_DROP: &str = "hotbolt_entry_state_serialize_drop";

/// Low level version of [`App`](App).
pub trait FfiApp {
	/// Creates a app. The app consists of mostly static code that is rarely changed.
	fn app_new() -> *mut c_void;

	/// Drops the app. Skipped when possible. Do not use this for side-effects.
	fn app_drop(app_ptr: *mut c_void);
}

impl<T: App> FfiApp for T {
	fn app_new() -> *mut c_void {
		let app = Box::new(T::new());
		Box::into_raw(app) as *mut c_void
	}

	fn app_drop(app_ptr: *mut c_void) {
		unsafe { Box::from_raw(app_ptr as *mut T) };
	}
}

/// See [`FfiApp::app_new`](FfiApp::app_new).
pub const ENTRY_APP_NEW: &str = "hotbolt_entry_app_new";

/// See [`FfiApp::app_drop`](FfiApp::app_drop).
pub const ENTRY_APP_DROP: &str = "hotbolt_entry_app_drop";

/// Low level version of [`AppVersion`](AppVersion).
pub trait FfiAppVersion {
	/// Returns the version identifying the app. This should be from static memory.
	fn app_version() -> FfiArray<'static, u8>;

	/// Returns true if the the given version is compatible with the current version.
	fn app_compatible(other: FfiArray<'static, u8>) -> bool;
}

impl<T: AppVersion<T = str>> FfiAppVersion for T {
	fn app_version() -> FfiArray<'static, u8> {
		T::version().into()
	}

	fn app_compatible(other: FfiArray<u8>) -> bool {
		T::is_compatible(unsafe { other.as_str_unchecked() })
	}
}

/// See [`FfiAppVersion::app_version`](FfiAppVersion::app_version).
pub const ENTRY_APP_VERSION: &str = "hotbolt_entry_app_version";

/// See [`FfiAppVersion::app_compatible`](FfiAppVersion::app_compatible).
pub const ENTRY_APP_COMPATIBLE: &str = "hotbolt_entry_app_compatible";
