use crate::common::{Deserializer, Serializer};

/// Whether the functionality is enabled.
///
/// The presence of this flag implies that uses of this object may be skipped because they don't do
/// anything. It is expressed as a separate trait because this is monomorphic and not available on
/// trait objects for compile-time optimizations purposes.
pub trait ServerEnabled {
	/// Whether this object has features enabled. Typically inlined.
	fn is_server_enabled() -> bool;
}

/// Base hot reload server functionality. All servers implement this.
///
/// See [`Server`](Server) for the full server implementation.
pub trait ServerBase<T: ?Sized>: ServerEnabled {
	/// Restarts both the client and application with an empty state.
	fn restart_hard(&self);

	/// Restarts the client and application with the given state.
	fn restart_hard_with<U: AsRef<T>>(&self, state: U);
}

/// Full hot reload server functionality.
pub trait Server<T: ?Sized>: ServerBase<T> {
	/// Restarts the application (not client) with an empty state.
	fn restart_soft(&self);

	/// Restarts the application (not client) with the given state.
	fn restart_soft_with<U: AsRef<T>>(&self, state: U);
}

/// The serializable, volatile state of the application.
///
/// See [`App`](App).
pub trait State {
	fn new() -> Self;
}

/// Helper trait pairing a state type with serialization/deserialization facilities.
pub trait StateConverter {
	type State: State;
	type Serializer: Serializer<Self::State>;
	type Deserializer: Deserializer<Self::State>;
}

/// Handles application versioning.
pub trait AppVersion {
	// TODO: Default to str. https://github.com/rust-lang/rust/issues/29661
	type T: ?Sized + PartialEq<Self::T> + 'static;

	/// Returns a static string indicating the version number of the application.
	fn version() -> &'static Self::T;

	/// Returns true if the given other version is compatible with this one.
	fn is_compatible(other: &Self::T) -> bool {
		Self::version() == other
	}
}

/// The non-serializable, non-volatile state of the application.
///
/// See [`State`](State).
pub trait App {
	fn new() -> Self;
}

// TODO: Won't need a macro when we have negative trait bounds.
// TODO: Maybe use infallible type instead of unit?
/// Creates a new type `$i` that is either `$t` or the unit type `()`.
macro_rules! MaybeUnit {
	(#[doc = $doc:expr], $i:ident, $t:ident) => {
		#[doc = $doc]
		pub trait $i {}
		impl<T: $t> $i for T {}
		impl $i for () {}
	};
}

MaybeUnit!(#[doc = "Type that is either [`App`](App) or a unit type indicating absence."], MaybeApp, App);
MaybeUnit!(#[doc = "Type that is either [`AppVersion`](AppVersion) or a unit type indicating absence."], MaybeAppVersion, AppVersion);

/// The main entry point for your application.
pub trait Run {
	type StateConverter: StateConverter;
	type App: MaybeApp;
	type AppVersion: MaybeAppVersion;

	/// Runs the application. Called in a loop, until the application is ready to shut down.
	fn run(
		app: &mut Self::App,
		server: impl Server<<Self::StateConverter as StateConverter>::State>,
		state: &mut <Self::StateConverter as StateConverter>::State,
	);
}

/// A main entry point for applications where you don't need non-volatile state.
///
/// See [`Run`](Run).
pub trait BaseRun {
	type StateConverter: StateConverter;

	/// See [`Run::run`](Run::run).
	fn run(
		server: impl ServerBase<<Self::StateConverter as StateConverter>::State>,
		state: &mut <Self::StateConverter as StateConverter>::State,
	);
}

impl<T: BaseRun> Run for T {
	type StateConverter = T::StateConverter;
	type App = ();
	type AppVersion = ();

	fn run(
		_app: &mut Self::App,
		server: impl Server<<Self::StateConverter as StateConverter>::State>,
		state: &mut <Self::StateConverter as StateConverter>::State,
	) {
		T::run(server, state)
	}
}
