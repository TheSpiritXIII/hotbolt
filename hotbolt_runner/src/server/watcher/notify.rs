use std::{path::Path, process, sync::mpsc::Sender};

use log::{debug, error, info};

use notify::{
	event::{Event, EventKind},
	RecommendedWatcher, RecursiveMode, Watcher,
};

use super::WatcherEvent;

pub struct NotifyWatcher;

impl NotifyWatcher {
	pub fn new() -> Self {
		Self
	}
}

impl super::Watcher for NotifyWatcher {
	fn run(&self, filepath: impl AsRef<Path>, sender: Sender<WatcherEvent>) -> Result<(), String> {
		let path = filepath.as_ref();
		if !path.is_file() {
			return Err(format!("Input `{}` must be a file", path.display()));
		}
		let dir = path.parent().ok_or_else(|| {
			return format!("Failed to get path directory of `{}`", path.display());
		})?;

		debug!("Started with library path: {}", path.display());

		let watcher_result: Result<RecommendedWatcher, _> =
			Watcher::new(move |res: Result<Event, _>| match res {
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
}
