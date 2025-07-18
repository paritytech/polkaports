use corevm_linker::ConfigureCargo;
use std::{path::Path, process::Command};
use tempfile::TempDir;

#[test]
fn build_hello_std() {
	build_hello(true);
}

#[test]
fn build_hello_no_std() {
	build_hello(false);
}

fn build_hello(std: bool) {
	let workdir = TempDir::new().unwrap();
	let target_dir = workdir.path().join("target");
	let crate_dir = workdir.path().join("hello");
	generate_crate(std, &crate_dir);
	let config = corevm_linker::Config { std, ..Default::default() };
	Command::new("cargo")
		.configure_cargo_for_corevm(&config)
		.unwrap()
		.env("CARGO_TARGET_DIR", &target_dir)
		.current_dir(&crate_dir)
		.arg("build")
		.checked_status()
		.unwrap();
	let target_name =
		if std { corevm_linker::STD_TARGET_NAME } else { corevm_linker::NO_STD_TARGET_NAME };
	let elf_path = target_dir.join(target_name).join("debug").join("hello");
	let elf = std::fs::read(&elf_path).unwrap();
	let mut config = polkavm_linker::Config::default();
	config.set_strip(true);
	let _linked =
		polkavm_linker::program_from_elf(config, &elf).expect("Failed to link pvm program:");
}

fn generate_crate(std: bool, dir: &Path) {
	let src = dir.join("src");
	std::fs::create_dir_all(&src).unwrap();
	let cargo_toml = if std {
		r#"[package]
name = "hello"
edition = "2021"
"#
	} else {
		r#"[package]
name = "hello"
edition = "2021"

[dependencies]
polkavm-derive = "0.26.0"
"#
	};
	std::fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
	let (code_file, code) = if std {
		let code = r#"
fn main() {
	println!("Hello, world!");
}
"#;
		("main.rs", code)
	} else {
		let code = r#"
#![no_std]
#![no_main]

#[polkavm_derive::polkavm_export]
pub extern "C" fn main() -> u64 {
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
"#;
		("main.rs", code)
	};
	std::fs::write(src.join(code_file), code).unwrap();
}

trait CheckStatus {
	fn checked_status(&mut self) -> Result<(), std::io::Error>;
}

impl CheckStatus for Command {
	fn checked_status(&mut self) -> Result<(), std::io::Error> {
		let status = self.status().map_err(|e| human_error(self, e))?;
		if status.success() {
			return Ok(());
		}
		Err(human_error(self, std::io::Error::other(format!("Exit status {status:?}"))))
	}
}

fn human_error(command: &Command, e: std::io::Error) -> std::io::Error {
	use std::fmt::Write;
	let mut message = String::new();
	let _ = write!(&mut message, "Failed to run");
	let _ = write!(&mut message, " {:?}", command.get_program().display());
	for arg in command.get_args() {
		let _ = write!(&mut message, " {:?}", arg.display());
	}
	let _ = write!(&mut message, ": {e}");
	std::io::Error::other(message)
}
