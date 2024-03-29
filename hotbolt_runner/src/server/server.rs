use std::{
	env, fs, io,
	net::TcpListener,
	path::Path,
	process::{self, Child, Command, Stdio},
	sync::mpsc::{self, TryRecvError},
};

use log::{error, info};

use super::watcher::{self, notify::NotifyWatcher, poll::PollWatcher, Watcher};
use crate::{
	common::{ClientMessage, ServerMessage, WatcherType},
	util::tcp,
	Cli,
};
use tcp::TcpPeer;
use watcher::WatcherEvent;

fn process_exit_code(process: &mut Child) -> Option<i32> {
	if let Ok(status) = process.try_wait() {
		if let Some(exit_code) = status {
			return Some(exit_code.code().unwrap_or(1));
		}
	}
	None
}

fn send<'a>(
	peer: &mut TcpPeer<'a, ClientMessage, ServerMessage>,
	process: &mut Child,
	request: ServerMessage,
) -> bool {
	if let Err(e) = peer.write(request) {
		error!("Error communicating with client: {}", e);
		error!("Retarting client process...");
		if let Err(_) = process.kill() {
			error!("Unable to kill client process. Continuing...");
		}
		false
	} else {
		true
	}
}

pub fn start<P1: AsRef<Path>, P2: AsRef<Path>>(
	lib_path: P1,
	lib_path_normalized: P2,
	address: &str,
	cli: Cli,
) {
	let (watcher_sender, watcher_receiver) = mpsc::channel();

	let watcher = match cli.watcher {
		WatcherType::Poll => {
			PollWatcher::new(std::time::Duration::from_secs(2)).run(&lib_path, watcher_sender)
		}
		WatcherType::Notify => NotifyWatcher::new().run(&lib_path, watcher_sender),
	};
	if let Err(e) = watcher {
		error!("{}", e);
		process::exit(1);
	}

	// TODO: This should live elsewhere.
	if let Err(e) = fs::copy(&lib_path, &lib_path_normalized) {
		error!("{}", e);
		error!("Unable to copy library file. Aborting");
	}

	let mut app_state = None;
	'spawn: loop {
		let listener = match TcpListener::bind(&address) {
			Ok(listener) => listener,
			Err(e) => {
				error!("Unable to start TCP listener on `{}`: {}", address, e);
				process::exit(1);
			}
		};
		if let Err(e) = listener.set_nonblocking(true) {
			error!("Unable to use non-blocking client socket connection: {}", e);
			process::exit(1);
		}

		info!("Spawning client process...");
		let app = match env::current_exe() {
			Ok(app) => app,
			Err(e) => {
				error!("Unable to get current exe path: {}", e);
				process::exit(1);
			}
		};
		let command = Command::new(app)
			.env("RUST_LOG", "hotbolt_runner=debug")
			.arg("--client")
			.arg(&cli.input)
			.args(&["--profile", &cli.profile])
			.args(&["--host", &cli.host])
			.args(&["--port", &cli.port])
			.stdout(Stdio::inherit())
			.stdin(Stdio::inherit())
			.stderr(Stdio::inherit())
			.spawn();
		let mut process = match command {
			Ok(process) => process,
			Err(e) => {
				error!("Unable to start process: {}", e);
				process::exit(1);
			}
		};

		// TODO: May need to clean child process when receiving an error.

		info!("Connecting to client...");
		let mut incoming_iter = listener.incoming().peekable();
		let stream = loop {
			if let Some(_) = incoming_iter.peek() {
				if let Some(incoming) = incoming_iter.next() {
					match incoming {
						Ok(stream) => break stream,
						Err(e) => {
							if e.kind() != io::ErrorKind::WouldBlock {
								error!("Unable to connect to client: {}", e);
								process::exit(1);
							}
						}
					};
				} else {
					error!("Unable to fetch incoming client connection");
					process::exit(1);
				}
			}

			// In case client dies before we have a chance to reconnect.
			if let Some(exit_code) = process_exit_code(&mut process) {
				// We don't want to retry because it will probably happen again.
				info!("Process exited with code: {}", exit_code);
				process::exit(1);
			}
		};

		info!("Connected");
		let mut message_stream = TcpPeer::<ClientMessage, ServerMessage>::from(&stream);
		if !send(
			&mut message_stream,
			&mut process,
			ServerMessage::Start(app_state.clone()),
		) {
			continue 'spawn;
		}

		let mut restarting = false;
		let mut file_exists: bool = true;

		loop {
			if restarting && file_exists {
				// TODO: This if statement only works if the client is listening (which it might not be).
				// We should kill after a timeout.
				let _ = process.kill();

				// TODO: This should live elsewhere.
				if let Err(e) = fs::copy(&lib_path, &lib_path_normalized) {
					error!("{}", e);
					error!("Unable to copy library file. Aborting");
				}
				// if !send(
				// 	&mut message_stream,
				// 	&mut process,
				// 	ServerMessage::Start(app_state.clone()),
				// ) {
				continue 'spawn;
				// }
			}

			match watcher_receiver.try_recv() {
				Ok(event) => match event {
					WatcherEvent::Created => {
						file_exists = true;
					}
					WatcherEvent::Changed => {
						if !send(&mut message_stream, &mut process, ServerMessage::GetState) {
							continue 'spawn;
						}
						restarting = true;
					}
					WatcherEvent::Destroyed => {
						file_exists = false;
					}
				},
				Err(e) => {
					if let TryRecvError::Disconnected = e {
						error!("Watcher disconnected");
						process::exit(1);
					}
				}
			}

			match message_stream.try_read() {
				Ok(maybe_message) => {
					if let Some(message) = maybe_message {
						match message {
							ClientMessage::Restart => {
								restarting = true;
								if !send(&mut message_stream, &mut process, ServerMessage::Close) {
									continue 'spawn;
								}
							}
							ClientMessage::SetState(client_state) => {
								app_state = client_state;
								if restarting {
									if !send(
										&mut message_stream,
										&mut process,
										ServerMessage::Close,
									) {
										continue 'spawn;
									}
								}
							}
						}
					}
				}
				Err(e) => {
					if e.kind() != io::ErrorKind::WouldBlock {
						if let Some(exit_code) = process_exit_code(&mut process) {
							info!("Process exited with code: {}", exit_code);
						} else {
							error!("Client connection lost: {}", e);
							if let Err(_) = process.kill() {
								error!("Unable to kill client process. Continuing...");
							}
						}
						continue 'spawn;
					}
				}
			}

			if let Some(exit_code) = process_exit_code(&mut process) {
				info!("Process exited with code: {}", exit_code);
				continue 'spawn;
			}
		}
	}

	// TODO: Gracefully shut down client.
}
