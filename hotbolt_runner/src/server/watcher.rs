use std::{path::Path, process, sync::mpsc::Sender, thread, time::Duration};

use log::{debug, error, info};
use notify::{
	event::{Event, EventKind},
	RecommendedWatcher, RecursiveMode, Watcher,
};

#[derive(Debug)]
pub enum WatcherEvent {
	Created,
	Changed,
	Destroyed,
}

pub fn watch_poll<P: AsRef<Path>>(
	filepath: P,
	sender: Sender<WatcherEvent>,
	interval: Duration,
) -> Result<(), String> {
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

pub fn watch_notify<P: AsRef<Path>>(
	filepath: P,
	sender: Sender<WatcherEvent>,
) -> Result<(), String> {
	let path = filepath.as_ref();
	if !path.is_file() {
		return Err(format!("Input `{}` must be a file", path.display()));
	}
	let dir = path
		.parent()
		.ok_or_else(|| return format!("Failed to get path directory of `{}`", path.display()))?;

	debug!("Started with library path: {}", path.display());

	let watcher_result: Result<RecommendedWatcher, _> =
		Watcher::new_immediate(move |res: Result<Event, _>| match res {
			Ok(event) => {
				let runner_event = match event.kind {
					EventKind::Create(_) => {
						info!("File was created");
						Some(WatcherEvent::Created)
					}
					EventKind::Modify(_) => Some(WatcherEvent::Changed),
					EventKind::Remove(_) => {
						info!("File was removed");
						Some(WatcherEvent::Destroyed)
					}
					_ => None,
				};
				if let Some(runner_event) = runner_event {
					if sender.send(runner_event).is_err() {
						error!("Unable to send runner event");
						process::exit(1);
					}
				}
			}
			Err(_) => {
				error!("Filesystem watcher error. Aborting");
				process::exit(1);
			}
		});

	if let Ok(mut watcher) = watcher_result {
		match watcher.watch(dir, RecursiveMode::NonRecursive) {
			Ok(_) => Ok(()),
			Err(_) => Err("Failed to attach filesystem watcher to file".to_string()),
		}
	} else {
		Err("Failed to instaniate filesystem watcher".to_string())
	}
}
