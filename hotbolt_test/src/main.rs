use std::{io, thread, time::Duration};

use log::info;

use crate::project::Builder;

mod project;

const EXAMPLE_CODE_BEFORE: &'static str = "
use hotbolt::hotbolt_entry_main;
#[hotbolt_entry_main]
fn main() {
	println!(\"Hello world!\");
	loop {}
}
";

const EXAMPLE_CODE_AFTER: &'static str = "
use hotbolt::hotbolt_entry_main;
#[hotbolt_entry_main]
fn main() {
	println!(\"Hello hotbolt!\");
	loop {}
}
";

macro_rules! try_block {
	($e:expr) => {
		(|| $e)()
	};
}

fn main() -> io::Result<()> {
	env_logger::init();

	let builder = Builder::new();
	builder.clean_all()?;

	let result = test_basic(&builder);

	info!("Done running all tests");
	builder.clean_all()?;
	result
}

fn test_basic(builder: &Builder) -> io::Result<()> {
	info!("Running basic test");
	let project = builder.build("basic")?;
	project.update(EXAMPLE_CODE_BEFORE)?;
	project.build()?;

	let mut child = project.hotbolt()?;
	let result = try_block! {{
		thread::sleep(Duration::from_secs(10));

		project.update(EXAMPLE_CODE_AFTER)?;
		project.build()?;

		thread::sleep(Duration::from_secs(10));
		Ok(())
	}};
	child.kill()?;
	result
}
