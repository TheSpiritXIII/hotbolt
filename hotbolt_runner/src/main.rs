use std::env;

fn main() {
	if let Some(lib_path) = env::args().skip(1).take(1).next() {
		println!("Library: {}", lib_path);
	} else {
		println!("Must specify library path");
	}
}
