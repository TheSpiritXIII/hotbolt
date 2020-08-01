#[cfg(target_os = "windows")]
pub fn library_format(name: &str) -> String {
	format!("{}.dll", name)
}

#[cfg(target_os = "macos")]
pub fn library_format(name: &str) -> String {
	format!("lib{}.dylib", name)
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn library_format(name: &str) -> String {
	format!("lib{}.so", name)
}
