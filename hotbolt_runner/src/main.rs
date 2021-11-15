pub mod client;
pub mod common;
pub mod server;
pub mod util;
#[cfg(test)]
mod test;

use std::process;

use common::Cli;
use log::{debug, error};

fn main() {
	env_logger::init();

	let cli = Cli::parse();

	let lib_path = cli.library_path().unwrap_or_else(|e| {
		error!("{}", e);
		error!("Unable to resolve file path. Aborting");
		process::exit(1);
	});

	let address = format!("{}:{}", &cli.host, &cli.port);

	let lib_path_normalized = lib_path.with_extension("hotbolt");
	if !cli.client {
		server::start(lib_path, lib_path_normalized, &address, cli);
	} else {
		debug!("Starting client with: {:?}", &lib_path_normalized);
		client::start(lib_path_normalized, &address);
	}
}
