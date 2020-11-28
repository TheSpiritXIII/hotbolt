use std::ffi::c_void;

use crate::{FfiArray, FfiArrayMut};

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

/// The version of the hotbolt runner that this library supports.
pub const RUNNER_VERSION: u8 = 0;

/// The internal hotbolt runner version this was written to support.
///
/// Signature: `() -> u8`
pub const ENTRY_VERSION: &str = "hotbolt_entry_version";

/// Low level version of [`Entry`](Entry) or [`App`](App).
pub trait FfiEntry {
	/// Runs the application. This is called in a loop.
	fn run(app_ptr: *mut c_void, server: FfiServer, state_ptr: *mut c_void);
}

/// See [`FfiEntry::run`](FfiState::run).
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

/// See [`FfiAppVersion::app_version`](FfiAppVersion::app_version).
pub const ENTRY_APP_VERSION: &str = "hotbolt_entry_app_version";

/// See [`FfiAppVersion::app_compatible`](FfiAppVersion::app_compatible).
pub const ENTRY_APP_COMPATIBLE: &str = "hotbolt_entry_app_compatible";
