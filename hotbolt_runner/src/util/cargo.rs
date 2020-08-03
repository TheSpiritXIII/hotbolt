use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::util::platform;

#[derive(Deserialize)]
struct Config {
	package: Option<Package>,
	lib: Option<Lib>,
}

#[derive(Deserialize)]
struct Package {
	name: Option<String>,
}

#[derive(Deserialize)]
struct Lib {
	name: Option<String>,
}

pub fn cargo_path<P: AsRef<Path>>(dir: P) -> Option<PathBuf> {
	let path = dir.as_ref().join("Cargo.toml");
	if path.is_file() {
		Some(path)
	} else {
		None
	}
}

pub fn cargo_lib_name<P: AsRef<Path>>(path: P) -> Result<String, String> {
	if let Ok(cargo_content) = fs::read_to_string(&path) {
		if let Ok(cargo_config) = toml::from_str::<Config>(&cargo_content) {
			cargo_config
				.lib
				.as_ref()
				.map(|lib| lib.name.as_ref())
				.flatten()
				.or_else(|| {
					cargo_config
						.package
						.as_ref()
						.map(|package| package.name.as_ref())
						.flatten()
				})
				.ok_or_else(|| {
					format!(
						"Unable to find library name in project `{}`",
						path.as_ref().display()
					)
				})
				.map(String::clone)
		} else {
			Err(format!(
				"Unable to parse Cargo.toml file in project `{}`",
				path.as_ref().display()
			))
		}
	} else {
		Err(format!(
			"Unable to read Cargo.toml file in project `{}`",
			path.as_ref().display()
		))
	}
}

pub fn cargo_target_path<P: AsRef<Path>>(path: &P) -> Result<PathBuf, String> {
	let target_local = path.as_ref().join("/target");
	if target_local.is_dir() {
		Ok(target_local)
	} else {
		let parent_path = path.as_ref().parent().ok_or_else(|| {
			format!(
				"Unable to find parent path of `{}`",
				path.as_ref().display()
			)
		})?;
		if cargo_path(parent_path).is_some() {
			let target_workspace = parent_path.join("target");
			if target_workspace.is_dir() {
				Ok(target_workspace)
			} else {
				Err(format!(
					"Invalid target location in `{}`",
					parent_path.display()
				))
			}
		} else {
			Err(format!("Invalid Cargo path in `{}`", parent_path.display()))
		}
	}
}

pub fn cargo_target_lib_path<P: AsRef<Path>>(dir: P, profile: &str) -> Result<PathBuf, String> {
	let cargo_local = cargo_path(&dir);
	if let Some(cargo_local_path) = cargo_local {
		let lib_name = cargo_lib_name(&cargo_local_path)?;
		let lib_filename = platform::library_format(&lib_name);
		let target_path = cargo_target_path(&dir)?;
		Ok(target_path.join(profile).join(lib_filename))
	} else {
		Err(format!(
			"Unable to get lib path of non-Cargo project directory `{}`",
			dir.as_ref().display()
		))
	}
}
