pub mod cargo;
pub mod platform;
pub mod runner;
pub mod watcher;

use std::sync::mpsc::channel;
use std::{
	mem,
	path::{Path, PathBuf},
	process,
};

use clap::Clap;
use log::error;

fn path_validator(input: &str) -> Result<(), String> {
	let path: &Path = input.as_ref();
	if !path.exists() {
		return Err("File does not exist.".to_owned());
	}
	Ok(())
}

#[derive(Clap)]
#[clap(version = "0.1")]
struct Opts {
	/// The directory of your Cargo project or file if using --file.
	#[clap(validator = path_validator)]
	input: String,

	/// Expects a library as opposed to Cargo project as input
	#[clap(short, long)]
	file: bool,

	/// The Cargo profile to use (when not using --file)
	#[clap(short, long, default_value = "debug", conflicts_with = "file")]
	profile: String,
}

impl Opts {
	fn library_path(&self) -> Result<PathBuf, String> {
		let path: &Path = self.input.as_ref();
		if !self.file {
			if path.is_dir() {
				cargo::cargo_target_lib_path(path, &self.profile)
			} else {
				Err(format!(
					"Must be Cargo project directory `{}`",
					path.display()
				))
			}
		} else {
			if path.is_file() {
				Ok(path.to_owned())
			} else {
				Err(format!("Must be a library project `{}`", path.display()))
			}
		}
	}
}

fn main() {
	env_logger::init();

	let opts: Opts = Opts::parse();

	let lib_path = opts.library_path().unwrap_or_else(|e| {
		error!("{}", e);
		error!("Unable to resolve file path. Aborting");
		process::exit(1);
	});

	let (sender, receiver) = channel();
	let watcher = match watcher::watch(&lib_path, sender.clone()) {
		Ok(watcher) => watcher,
		Err(e) => {
			error!("{}", e);
			process::exit(1);
		}
	};

	// We need watcher in scope for the entire application lifecycle.
	// We don't want it to deallocate and stop listening to events.
	mem::forget(watcher);

	runner::run(lib_path, sender, receiver);
}
