#[cfg(not(feature = "hotbolt_erase"))]
mod main;
#[cfg(not(feature = "hotbolt_erase"))]
pub use main::*;
