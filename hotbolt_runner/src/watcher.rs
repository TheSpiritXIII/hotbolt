use std::{path::Path, process, sync::mpsc::Sender};

use log::{debug, info, error};
use notify::event::{Event, EventKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::runner::Event as RunnerEvent;

pub fn watch(filepath: &str, sender: Sender<RunnerEvent>) -> Result<impl Sized, String> {
	let path = Path::new(&filepath);
	if !path.is_file() {
		return Err(format!("Input `{}` must be a file", filepath));
	}
	let dir = path
		.parent()
		.ok_or_else(|| return format!("Failed to get path directory of `{}`", filepath))?;

	debug!("Started with library path: {}", filepath);

	let watcher_result: Result<RecommendedWatcher, _> =
		Watcher::new_immediate(move |res: Result<Event, _>| match res {
			Ok(event) => {
				let runner_event = match event.kind {
					EventKind::Create(_) => {
						info!("File was created");
						Some(RunnerEvent::Start)
					},
					EventKind::Modify(_) => Some(RunnerEvent::Reload),
					EventKind::Remove(_) => {
						info!("File was removed");
						None
					}
					_ => None,
				};
				if let Some(runner_event) = runner_event {
					if let Err(_) = sender.send(runner_event) {
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
			Ok(_) => Ok(watcher),
			Err(_) => Err(format!("Failed to attach filesystem watcher to file")),
		}
	} else {
		Err(format!("Failed to instaniate filesystem watcher"))
	}
}
