use std::{path::Path, process, sync::mpsc::Sender, thread, time::Duration};

use log::error;

use super::WatcherEvent;

pub struct PollWatcher {
	interval: Duration,
}

impl PollWatcher {
	pub fn new(interval: Duration) -> Self {
		Self { interval }
	}
}

impl super::Watcher for PollWatcher {
	fn run(&self, filepath: impl AsRef<Path>, sender: Sender<WatcherEvent>) -> Result<(), String> {
		let path = filepath.as_ref().to_owned();
		if !path.is_file() {
			return Err(format!("Input `{}` must be a file", path.display()));
		}

		let metadata = path
			.metadata()
			.map_err(|_| format!("Input `{}` could not get metadata", path.display()))?;
		let last_modified_time = metadata.modified().map_err(|_| {
			format!(
				"Input `{}` could not get last modified time",
				path.display()
			)
		})?;

		// TODO: Don't clone interval.
		let interval = self.interval.clone();
		thread::spawn(move || {
			let mut last_modified = Some(last_modified_time);
			loop {
				let event = if let Ok(metadata) = path.metadata() {
					if let Ok(modified_time) = metadata.modified() {
						if let Some(last_modified_time) = last_modified {
							if last_modified_time != modified_time {
								last_modified = Some(modified_time);
								Some(WatcherEvent::Changed)
							} else {
								None
							}
						} else {
							last_modified = Some(modified_time);
							Some(WatcherEvent::Created)
						}
					} else {
						Some(WatcherEvent::Destroyed)
					}
				} else {
					Some(WatcherEvent::Destroyed)
				};

				if let Some(runner_event) = event {
					error!("Sending message: {:?}", runner_event);
					if sender.send(runner_event).is_err() {
						error!("Unable to send runner event");
						process::exit(1);
					}
				}

				thread::sleep(interval);
			}
		});
		Ok(())
	}
}
