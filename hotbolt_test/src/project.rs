use std::{
	env::temp_dir,
	fs::{create_dir, remove_dir_all, OpenOptions},
	io::{self, Error, ErrorKind, Seek, SeekFrom, Write},
	path::{Path, PathBuf},
	process::{Child, Command},
};

use log::info;

fn root_directory() -> PathBuf {
	let dir = env!("CARGO_MANIFEST_DIR");
	let mut path = PathBuf::from(dir);
	path.pop();
	path
}

fn hotbolt_project_dir() -> PathBuf {
	let mut dir = root_directory();
	dir.push("hotbolt");
	dir
}

pub struct Builder {
	dir: PathBuf,
}

impl Builder {
	pub fn new() -> Self {
		Self {
			dir: temp_dir().join("hotbolt"),
		}
	}

	pub fn build(&self, name: &str) -> io::Result<Project> {
		Project::setup(&self.dir, name)
	}

	pub fn clean_all(&self) -> io::Result<()> {
		if self.dir.exists() {
			info!("Removing directory: {}", self.dir.display());
			remove_dir_all(&self.dir)?;
		}
		Ok(())
	}
}

pub struct Project {
	dir: PathBuf,
	code: PathBuf,
}

impl Project {
	fn setup(dir: impl AsRef<Path>, name: &str) -> io::Result<Self> {
		if !Path::exists(dir.as_ref()) {
			info!("Creating dir: {}", Path::display(dir.as_ref()));
			create_dir(&dir)?;
		}

		info!("Setting up project: {}", name);
		let mut command = Command::new("cargo");
		let status = command
			.arg("new")
			.arg("--lib")
			.arg(name)
			// .arg("--config")
			// .arg("lib.crate-type=[\"cdylib\"]")
			.current_dir(&dir)
			.status()?;
		if !status.success() {
			Error::new(ErrorKind::Other, "Cargo init failed");
		}

		let project_dir = dir.as_ref().join(name);
		if !project_dir.exists() {
			Error::new(ErrorKind::NotFound, "Unable to find project path");
		}

		info!("Editing Cargo.toml");
		let cargo_config = project_dir.join("Cargo.toml");
		let mut file = OpenOptions::new()
			.write(true)
			.create(false)
			.truncate(false)
			.open(&cargo_config)?;
		file.seek(SeekFrom::End(0))?;
		writeln!(
			file,
			"hotbolt = {{ path = \"{}\"}}\n",
			hotbolt_project_dir()
				.display()
				.to_string()
				.replace("\\", "\\\\")
		)?;
		writeln!(file, "[lib]\ncrate-type=[\"cdylib\"]")?;

		let code = project_dir.join("src/lib.rs");
		if !code.exists() {
			Error::new(ErrorKind::NotFound, "Unable to find lib.rs file");
		}

		Ok(Self {
			dir: project_dir,
			code,
		})
	}

	pub fn update(&self, content: &str) -> io::Result<()> {
		info!("Updating lib.rs");
		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(&self.code)?;
		file.write_all(content.as_bytes())?;
		Ok(())
	}

	pub fn build(&self) -> io::Result<()> {
		info!("Building project");
		let mut command = Command::new("cargo");
		let status = command.arg("build").current_dir(&self.dir).status()?;
		if !status.success() {
			Error::new(ErrorKind::Other, "Cargo build failed");
		}
		Ok(())
	}

	pub fn hotbolt(&self) -> io::Result<Child> {
		info!("Starting hotbolt");
		let mut command = Command::new("cargo");
		let child = command
			.arg("run")
			.arg("--bin")
			.arg("hotbolt-runner")
			.arg(self.dir.display().to_string())
			.current_dir(root_directory())
			// .stdout(Stdio::piped())
			// .stderr(Stdio::piped())
			.spawn()?;
		Ok(child)
	}
}
