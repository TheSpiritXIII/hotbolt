use std::{io, time::Duration};

use log::info;

use crate::project::Builder;

mod project;
mod reload;

const EXAMPLE_CODE_BEFORE: &'static str = "
use hotbolt::hotbolt_entry_main;
#[hotbolt_entry_main]
fn main() {
	println!(\"Hello world!\");
	loop {}
}
";

const EXAMPLE_OUT_BEFORE: &'static str = "Hello world!";

const EXAMPLE_CODE_AFTER: &'static str = "
use hotbolt::hotbolt_entry_main;
#[hotbolt_entry_main]
fn main() {
	println!(\"Hello hot reload!\");
	loop {}
}
";

const EXAMPLE_OUT_AFTER: &'static str = "Hello hot reload!";

#[tokio::main]
async fn main() -> io::Result<()> {
	env_logger::init();

	let builder = Builder::new();
	builder.clean_all()?;

	let result = test_basic(&builder);

	info!("Done running all tests");
	builder.clean_all()?;
	result.await
}

async fn test_basic(builder: &Builder) -> io::Result<()> {
	info!("Running basic test");
	let project = builder.build("basic")?;
	project.update(EXAMPLE_CODE_BEFORE)?;
	project.build()?;

	let mut reload = project
		.hot_reload()
		.timeout(Duration::from_secs(60))
		.expect(EXAMPLE_OUT_BEFORE)
		.await?;
	let result = async {
		project.update(EXAMPLE_CODE_AFTER)?;
		project.build()?;
		reload.expect(EXAMPLE_OUT_AFTER).await?;
		Ok(())
	}
	.await;
	reload.take().kill().await?;

	result
}
