use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
use std::{io, marker::PhantomData, net::TcpStream, thread};

// TODO: Does mio, tokio or async_std provide anything that could help me replace this?
pub fn io_blocking<T, F: FnMut() -> io::Result<T>>(mut f: F) -> io::Result<T> {
	loop {
		let result = f();
		if let Err(e) = &result {
			if e.kind() == io::ErrorKind::WouldBlock {
				thread::yield_now();
				continue;
			}
		}
		return result;
	}
}

// TODO: Replace this entire thing with Servo's channels if they ever add Windows support.
#[derive(Copy, Clone)]
pub struct TcpPeer<'a, TInput: DeserializeOwned, TOutput: Serialize> {
	stream: &'a TcpStream,
	phantom_input: PhantomData<TInput>,
	phantom_output: PhantomData<TOutput>,
}

impl<'a, TInput: DeserializeOwned, TOutput: Serialize> TcpPeer<'a, TInput, TOutput> {
	pub fn from(stream: &'a TcpStream) -> Self {
		Self {
			stream,
			phantom_input: PhantomData,
			phantom_output: PhantomData,
		}
	}

	pub fn read(&mut self) -> io::Result<TInput> {
		let mut length_buf: [u8; 8] = [0; 8];
		self.read_exact_blocking(&mut length_buf)?;

		let length = usize::from_le_bytes(length_buf);
		let mut content_buf = vec![0; length];
		self.read_exact_blocking(&mut content_buf)?;

		let buf = &content_buf[..];
		let data: TInput = bincode::deserialize(buf).unwrap();
		Ok(data)
	}

	pub fn try_read(&mut self) -> io::Result<Option<TInput>> {
		if self.stream.peek(&mut [0; 1])? > 0 {
			self.read().map(Option::Some)
		} else {
			Ok(None)
		}
	}

	pub fn write(&mut self, data: TOutput) -> io::Result<()> {
		let encoded = bincode::serialize(&data).unwrap();
		self.write_blocking(&encoded.len().to_le_bytes())?;
		self.write_blocking(&encoded)?;
		Ok(())
	}

	fn read_exact_blocking(&mut self, buf: &mut [u8]) -> io::Result<()> {
		io_blocking(|| self.stream.read_exact(buf))
	}

	fn write_blocking(&mut self, buf: &[u8]) -> io::Result<usize> {
		io_blocking(|| self.stream.write(buf))
	}
}
