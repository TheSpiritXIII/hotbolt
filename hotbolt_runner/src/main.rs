pub mod client;
pub mod common;
pub mod server;
pub mod util;

use std::process;

use common::Cli;
use log::error;

fn main() {
	env_logger::init();

	let cli = Cli::parse();

	let lib_path = cli.library_path().unwrap_or_else(|e| {
		error!("{}", e);
		error!("Unable to resolve file path. Aborting");
		process::exit(1);
	});

	let address = format!("{}:{}", &cli.host, &cli.port);

	if !cli.client {
		server::start(lib_path, &address, cli);
	} else {
		client::start(lib_path, &address);
	}
}
