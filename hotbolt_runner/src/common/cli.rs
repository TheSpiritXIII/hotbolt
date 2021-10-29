use std::{
	path::{Path, PathBuf},
	str::FromStr,
};

use clap::Parser;

use crate::util::cargo;

fn path_validator(input: &str) -> Result<(), String> {
	let path: &Path = input.as_ref();
	if !path.exists() {
		return Err(format!("File `{}` does not exist.", input));
	}
	Ok(())
}
#[derive(Parser)]
#[clap(version = "0.1")]
pub struct Cli {
	/// The directory of your Cargo project or file if using --file.
	#[clap(validator = path_validator)]
	pub input: String,

	/// Expects a library as opposed to Cargo project as input
	#[clap(short, long)]
	pub file: bool,

	/// The Cargo profile to use (when not using --file)
	#[clap(long, default_value = "debug", conflicts_with = "file")]
	pub profile: String,

	/// The hostname for the server/client connection.
	#[clap(long, default_value = "localhost")]
	pub host: String,

	/// The port for the server/client connection.
	#[clap(long, default_value = "49152")]
	pub port: String,

	// TODO: Need a way to pass polling duration.
	/// The server watcher type.
	#[clap(long, default_value = "poll")]
	pub watcher: WatcherType,

	/// Whether the application is started in client mode or server mode.
	#[clap(long)]
	pub client: bool,
}

pub enum WatcherType {
	Poll,
	Notify,
}

impl FromStr for WatcherType {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"poll" => Ok(WatcherType::Poll),
			"notify" => Ok(WatcherType::Notify),
			_ => Err("no match"),
		}
	}
}

impl Cli {
	pub fn parse() -> Self {
		Parser::parse()
	}

	pub fn library_path(&self) -> Result<PathBuf, String> {
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
