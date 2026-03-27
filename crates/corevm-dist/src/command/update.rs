use anyhow::anyhow;

use std::process::{Command, Stdio};

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

pub fn update() -> anyhow::Result<()> {
	self_update()?;
	// This call returns only on error.
	execute_install()?;
	Ok(())
}

fn self_update() -> anyhow::Result<()> {
	let cargo_env = std::env::var_os("CARGO")
		.map(|cargo| format!("env CARGO={cargo:?} "))
		.unwrap_or_default();
	let command_string = format!("{cargo_env}cargo install {CRATE_NAME}",);
	println!(
		"corevm-dist will run the following command: \n\n    {command_string}\n\nIs this okay? [y/N]"
	);
	loop {
		let mut answer = String::new();
		std::io::stdin().read_line(&mut answer)?;
		if answer.trim().to_lowercase() == "y" {
			break;
		}
	}
	let mut cargo = Command::new("cargo");
	cargo.stdin(Stdio::null());
	cargo.args(["install", CRATE_NAME]);
	let status = cargo.status()?;
	if !status.success() {
		return Err(anyhow!("Failed to run `{command_string}`: non-zero exit status"));
	}
	Ok(())
}

fn execute_install() -> anyhow::Result<()> {
	let exe = std::env::current_exe().map_err(|e| {
		anyhow!(
			"Failed to find the current executable ({e}). \
            Please run {CRATE_NAME} install yourself."
		)
	})?;
	let mut corevm_dist = Command::new(exe);
	corevm_dist.stdin(Stdio::null());
	corevm_dist.arg("install");
	execute(corevm_dist)
}

#[cfg(unix)]
fn execute(mut command: Command) -> anyhow::Result<()> {
	use std::os::unix::process::CommandExt;
	let error = command.exec();
	Err(anyhow!("Failed to execute `{CRATE_NAME} install`: {error}"))
}

#[cfg(not(unix))]
fn execute(mut command: Command) -> anyhow::Result<()> {
	let status = command.status()?;
	std::process::exit(status.code().unwrap_or(1));
}
