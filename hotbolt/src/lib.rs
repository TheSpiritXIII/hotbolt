pub use hotbolt_ffi::Server;
pub use hotbolt_macro::hotbolt_entry_main;

pub mod internal {
	pub use hotbolt_ffi::{FfiServer, SizedCharArray};
}
