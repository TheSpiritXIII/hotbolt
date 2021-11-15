use std::{
	io::{self, Error, ErrorKind},
	path::Path,
	process::Stdio,
	sync::{Arc, Mutex},
	time::Duration,
};

use log::{error, info};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::{Child, Command},
	sync::oneshot,
	time::timeout,
};

use super::project::root_directory;

pub struct HotReloadCommand {
	command: Command,
	duration: Duration,
}

impl HotReloadCommand {
	pub fn new(dir: impl AsRef<Path>) -> Self {

		let mut command = tokio::process::Command::new("cargo");
		command
			.arg("run")
			.arg("--bin")
			.arg("hotbolt-runner")
			.arg(dir.as_ref().display().to_string())
			.current_dir(root_directory())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			// TODO: revisit this -- not all terminals support this?
			.env("RUST_LOG_STYLE", "always");

		Self {
			command,
			duration: Duration::ZERO,
		}
	}

	pub fn timeout(mut self, duration: Duration) -> Self {
		self.duration = duration;
		self
	}

	pub async fn expect(self, text: &'static str) -> io::Result<HotReload> {
		HotReload::new(self, text).await
	}
}

pub struct HotReload {
	child: Child,
	duration: Duration,
	search: Arc<Mutex<Option<(&'static str, oneshot::Sender<io::Result<()>>)>>>,
}

impl HotReload {
	async fn new(mut reload: HotReloadCommand, text: &'static str) -> io::Result<Self> {
		let (out_sender, out_receiver) = oneshot::channel();
		let search = Arc::new(Mutex::new(Some((text, out_sender))));

		info!("Starting hot reload");
		let mut child = reload.command.spawn()?;

		let stdout = child
			.stdout
			.take()
			.ok_or_else(|| Error::new(ErrorKind::Other, "Unable to get hot reload stdout"))?;
		let stderr = child
			.stderr
			.take()
			.ok_or_else(|| Error::new(ErrorKind::Other, "Unable to get hot reload stderr"))?;

		let mut out_reader = BufReader::new(stdout).lines();
		let mut err_reader = BufReader::new(stderr).lines();

		let mut reload = HotReload {
			child,
			duration: reload.duration,
			search: search.clone(),
		};

		tokio::spawn(async move {
			loop {
				match out_reader.next_line().await {
					Ok(Some(line)) => {
						println!("{}", line);
						let mut search_lock = search.lock().expect("Unable to retrieve lock");
						if let Some((text, _)) = search_lock.as_ref() {
							if line.contains(text) {
								if let Some((_, sender)) = search_lock.take() {
									if let Err(_) = sender.send(Ok(())) {
										error!("Failed to send search result match");
									}
								}
							}
						}
					}
					Ok(None) => {
						break;
					}
					Err(_) => {
						error!("Failed to read stderr");
						break;
					}
				}
			}

			let mut search_lock = search.lock().expect("Unable to retrieve lock");
			if let Some((_, sender)) = search_lock.take() {
				sender
					.send(Err(io::Error::new(ErrorKind::Other, "Process quit")))
					.expect("Unable to send process quit error");
			}
		});

		tokio::spawn(async move {
			loop {
				match err_reader.next_line().await {
					Ok(Some(line)) => {
						println!("{}", line);
					}
					Ok(None) => {
						break;
					}
					Err(_) => {
						error!("Failed to read stderr");
						break;
					}
				}
			}
		});

		reload.wait(out_receiver).await.map(|_| reload)
	}

	pub async fn expect(&mut self, text: &'static str) -> io::Result<()> {
		let (out_sender, out_receiver) = oneshot::channel();

		let mut search_lock = self
			.search
			.lock()
			.map_err(|_| io::Error::new(ErrorKind::Other, "Search lock failed"))?;
		if search_lock.is_some() {
			return Err(io::Error::new(
				ErrorKind::Other,
				"Can only search for one text at a time",
			));
		}
		search_lock.replace((text, out_sender));
		drop(search_lock);

		self.wait(out_receiver).await
	}

	async fn wait(&mut self, receiver: oneshot::Receiver<io::Result<()>>) -> io::Result<()> {
		if let Ok(value) = timeout(self.duration, receiver).await {
			value.map_err(|_| io::Error::new(ErrorKind::Other, "Process listener failed"))?
		} else {
			Err(io::Error::new(ErrorKind::Other, "Timeout failed"))
		}
	}

	pub fn take(self) -> tokio::process::Child {
		self.child
	}
}
