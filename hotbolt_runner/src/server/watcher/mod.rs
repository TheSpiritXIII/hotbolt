use std::{path::Path, sync::mpsc::Sender};

pub mod notify;
pub mod poll;

#[derive(Debug)]
pub enum WatcherEvent {
	Created,
	Changed,
	Destroyed,
}

pub trait Watcher {
	fn run(&self, filepath: impl AsRef<Path>, sender: Sender<WatcherEvent>) -> Result<(), String>;
}
