use std::{
	io,
	mem,
	net::TcpStream,
	path::Path,
	process,
	sync::{
		atomic::{AtomicBool, Ordering},
		mpsc,
		Arc,
		Mutex,
		RwLock,
	},
	thread,
};

use log::{debug, error, info};

use super::runner;
use crate::{
	common::{ClientMessage, ServerMessage},
	util::tcp,
};
use runner::{HotboltLib, SenderServer};
use tcp::TcpPeer;

pub fn start<P: AsRef<Path>>(lib_path: P, address: &str) {
	let (sender, receiver) = mpsc::channel();

	debug!("Connecting to server...");
	let stream = match TcpStream::connect(&address) {
		Ok(stream) => stream,
		Err(e) => {
			error!("Unable to connect to server socket on `{}`: {}", address, e);
			process::exit(1);
		}
	};
	if let Err(e) = stream.set_nonblocking(true) {
		error!("Unable to use non-blocking server socket connection: {}", e);
		process::exit(1);
	}

	let library: Arc<RwLock<Option<HotboltLib>>> = Arc::new(RwLock::new(None));
	let state: Arc<Mutex<Option<Box<[u8]>>>> = Arc::new(Mutex::new(None));
	let loaded = Arc::new(AtomicBool::new(false));

	let library_thread = library.clone();
	let state_thread = state.clone();
	let loaded_thread = loaded.clone();
	thread::spawn(move || {
		let mut message_stream = TcpPeer::<ServerMessage, ClientMessage>::from(&stream);
		loop {
			let send = |peer: &mut TcpPeer<ServerMessage, ClientMessage>, message| {
				if let Err(e) = peer.write(message) {
					error!("Error communicating with server: {}", e);
				}
			};

			let get_state = || {
				let library_lock = library_thread.read().unwrap();
				let library = library_lock.as_ref().unwrap();
				let state = library.state().unwrap().state();
				state
			};

			if let Ok(event) = receiver.try_recv() {
				match event {
					runner::SenderEvent::Restart => {
						send(&mut message_stream, ClientMessage::SetState(None));
						send(&mut message_stream, ClientMessage::Restart);
					}
					runner::SenderEvent::Reload => {
						send(
							&mut message_stream,
							ClientMessage::SetState(Some(get_state())),
						);
						send(&mut message_stream, ClientMessage::Restart);
					}
					runner::SenderEvent::ReloadWith(_) => {}
				}
			}

			match message_stream.try_read() {
				Ok(data) => {
					if let Some(event) = data {
						match event {
							ServerMessage::GetState => {
								send(
									&mut message_stream,
									ClientMessage::SetState(Some(get_state())),
								);
							}
							ServerMessage::Start(app_state) => {
								let mut sl = state_thread.lock().expect("hi");
								let _ = mem::replace(&mut *sl, app_state);
								loaded_thread.store(true, Ordering::Relaxed);
							}
							ServerMessage::Close => {
								// TODO: In the future, exit gracefully.
								process::exit(1);
							}
						}
					}
				}
				Err(e) => {
					if e.kind() != io::ErrorKind::WouldBlock {
						error!("Error communicating with server: {}", e);
						process::exit(1);
					}
				}
			}
		}
	});

	let server = SenderServer { sender };
	loop {
		while !loaded.load(Ordering::Relaxed) {
			thread::yield_now();
		}

		let load_error;
		match HotboltLib::load(&lib_path) {
			Ok(lib) => {
				library.write().unwrap().replace(lib);
				info!("Successfully loaded library");

				let library_lock = library.as_ref().read().unwrap();
				let library = library_lock.as_ref().unwrap();
				match library.symbols() {
					Ok(symbols) => {
						let state = state.lock().unwrap();
						let value = state.as_ref().map(|x| x.as_ref()).unwrap_or(&[]);
						symbols.run(&server, value);
						load_error = false;
					}
					Err(err) => {
						error!("{}", err);
						load_error = true;
					}
				}
			}
			Err(err) => {
				error!("{}", err);
				load_error = true;
			}
		}

		if load_error {
			error!("Due to previous failure, waiting for restart confirmation...");
		}
	}
}
