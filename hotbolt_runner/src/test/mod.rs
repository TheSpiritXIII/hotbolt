mod project;
mod reload;

use std::{future::Future, io, sync::Once, time::Duration};

use log::info;

use project::{Builder, Project};

static INIT: Once = Once::new();

fn setup() {
	INIT.call_once(|| {
		env_logger::init();

		let builder = Builder::new();
		builder.clean_all().expect("Unable to clean temp directory");
	});
}

async fn test<R: Future<Output = io::Result<()>>>(
	name: &str,
	test_fn: impl FnOnce(Project) -> R,
) -> io::Result<()> {
	setup();

	let builder = Builder::new();
	info!("Running test: `{}`", name);
	let project = builder.build(name)?;
	let result = test_fn(project).await;

	info!("Finished test: `{}`", name);
	result
}

const TEST_HARD_REBUILD_CODE_BEFORE: &'static str = "
use hotbolt::hotbolt_entry_main;
#[hotbolt_entry_main]
fn main() {
	println!(\"Hello world!\");
	loop {}
}
";

const TEST_HARD_REBUILD_OUT_BEFORE: &'static str = "Hello world!";

const TEST_HARD_RELOAD_CODE_AFTER: &'static str = "
use hotbolt::hotbolt_entry_main;
#[hotbolt_entry_main]
fn main() {
	println!(\"Hello hot reload!\");
	loop {}
}
";

const TEST_HARD_RELOAD_OUT_AFTER: &'static str = "Hello hot reload!";

#[tokio::test]
async fn test_hard_rebuild() -> io::Result<()> {
	test("hard_rebuild", hard_rebuild).await
}

async fn hard_rebuild(project: Project) -> io::Result<()> {
	project.update(TEST_HARD_REBUILD_CODE_BEFORE)?;
	project.build()?;

	let mut reload = project
		.hot_reload()
		.timeout(Duration::from_secs(60))
		.expect(TEST_HARD_REBUILD_OUT_BEFORE)
		.await?;
	let result = async {
		project.update(TEST_HARD_RELOAD_CODE_AFTER)?;
		project.build()?;
		reload.expect(TEST_HARD_RELOAD_OUT_AFTER).await?;
		Ok(())
	}
	.await;
	reload.take().kill().await?;

	result
}

const TEST_HARD_MANUAL_CODE_BEFORE: &'static str = "
use hotbolt::{hotbolt_entry_main, Server};
#[hotbolt_entry_main]
fn main(server: impl Server) {
	println!(\"Hello world!\");
	std::thread::sleep(std::time::Duration::from_secs(3));
	server.restart();
}
";

const TEST_HARD_MANUAL_OUT_BEFORE: &'static str = "Hello world!";

const TEST_HARD_MANUAL_CODE_AFTER: &'static str = "
use hotbolt::{hotbolt_entry_main, Server};
#[hotbolt_entry_main]
fn main(server: impl Server) {
	println!(\"Hello hot reload!\");
	std::thread::sleep(std::time::Duration::from_secs(3));
	server.restart();
}
";

const TEST_HARD_MANUAL_OUT_AFTER: &'static str = "Hello hot reload!";

// TODO: Ignore this test until we fix it.
#[tokio::test]
#[ignore]
async fn test_hard_manual() -> io::Result<()> {
	test("hard_manual", hard_manual).await
}

async fn hard_manual(project: Project) -> io::Result<()> {
	project.update(TEST_HARD_MANUAL_CODE_BEFORE)?;
	project.build()?;

	let mut reload = project
		.hot_reload()
		.timeout(Duration::from_secs(60))
		.expect(TEST_HARD_MANUAL_OUT_BEFORE)
		.await?;
	let result = async {
		reload.expect(TEST_HARD_MANUAL_OUT_BEFORE).await?;

		project.update(TEST_HARD_MANUAL_CODE_AFTER)?;
		project.build()?;
		reload.expect(TEST_HARD_MANUAL_OUT_AFTER).await?;
		reload.expect(TEST_HARD_MANUAL_OUT_AFTER).await?;
		Ok(())
	}
	.await;
	reload.take().kill().await?;

	result
}

// TODO: Hard-reload: State.
// TODO: Soft-reload: Restart.
// TODO: Soft-reload: Manual.
// TODO: Soft-reload: Versioning.
