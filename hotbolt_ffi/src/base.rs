/// Whether the functionality is enabled. The presence of this flag implies that uses of this object may be skipped because they don't do anything..
///
/// Expressed as a separate trait because this is monomorphic and not available on trait objects for compile-time optimizations purposes.
pub trait ServerEnabled {
	/// Whether this object has features enabled. Typically inlined.
	fn is_server_enabled() -> bool;
}

/// Base hot reload server functionality. All servers implement this. See [`Server`](Server) for the full server implementation.
pub trait ServerBase<T: ?Sized>: ServerEnabled {
	/// Restarts both the client and application with an empty state.
	fn restart_hard(&self);

	// Restarts the client and application with the given state.
	fn restart_hard_with<U: AsRef<T>>(&self, state: U);
}

/// Full hot reload server functionality.
pub trait Server<T: ?Sized>: ServerBase<T> {
	/// Restarts the application (not client) with an empty state.
	fn restart_soft(&self);

	// Restarts the application (not client) with the given state.
	fn restart_soft_with<U: AsRef<T>>(&self, state: U);
}

/// The serializable state of the application. See [`App`](App).
pub trait State {
	fn new() -> Self;
}

/// Handles all application versioning facilities.
pub trait Version {
	// TODO: Default to str.
	type T: PartialEq<Self::T> + 'static;

	/// Returns a static string indicating the version number of the application.
	fn version() -> &'static Self::T;

	/// Returns true if the given other version is compatible with this one.
	fn is_compatible(other: &Self::T) -> bool {
		Self::version() == other
	}
}

/// The non-serializable state of the application. See [`State`](State).
pub trait App {
	fn new() -> Self;
}
