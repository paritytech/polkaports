use jam_codec::{Compact, Decode, Encode};
use std::{
	borrow::Cow,
	fs,
	io::{Error, ErrorKind},
	path::{Path, PathBuf},
	process::Command,
};

pub fn rust_target_file() -> Result<PathBuf, Error> {
	let sysroot = sysroot_dir()?;
	Ok(sysroot.join(TARGET_FILE))
}

pub fn sysroot_dir() -> Result<PathBuf, Error> {
	let sysroot = dirs::cache_dir()
		.unwrap_or_else(|| {
			let tmp = std::env::temp_dir();
			log::warn!("Using {tmp:?} as cache directory");
			tmp
		})
		.join(env!("CARGO_PKG_NAME"))
		.join(env!("CARGO_PKG_VERSION"));
	for (path, contents) in SYSROOT.iter() {
		let contents: Cow<'_, [u8]> = if path == &TARGET_FILE {
			// Interpolate @SYSROOT@ in the target file.
			let contents = std::str::from_utf8(contents).unwrap();
			Cow::Owned(contents.replace("@SYSROOT@", &sysroot.display().to_string()).into())
		} else {
			Cow::Borrowed(contents)
		};
		let path = sysroot.join(path);
		sync_file(&path, contents.as_ref())?;
	}
	// Write cargo config.
	sync_file(
		sysroot.join("std.toml"),
		format!(
			r#"[build]
target = "{}"
rustflags = ["-Cpanic=abort"]

[unstable]
build-std = ["core", "alloc", "std", "panic_abort"]
build-std-features = ["panic_immediate_abort"]
"#,
			sysroot.join(TARGET_FILE).display(),
		),
	)?;
	sync_file(
		sysroot.join("no_std.toml"),
		format!(
			r#"[build]
target = "{}"
rustflags = ["-Cpanic=abort"]

[unstable]
build-std = ["core", "alloc"]
"#,
			polkavm_linker::target_json_64_path().map_err(Error::other)?.display(),
		),
	)?;
	Ok(sysroot)
}

fn sync_file<P: AsRef<Path>, C: AsRef<[u8]>>(file: P, contents: C) -> Result<(), Error> {
	let file = file.as_ref();
	let is_stale = match fs::metadata(file) {
		Ok(metadata) => metadata.len() != contents.as_ref().len() as u64,
		Err(e) if e.kind() == ErrorKind::NotFound => true,
		Err(e) => {
			log::warn!("Failed to get metadata of {file:?}: {e}. Overwriting the file.");
			let _ = fs::remove_file(file);
			true
		},
	};
	if !is_stale {
		return Ok(());
	}
	if let Some(parent) = file.parent() {
		fs::create_dir_all(parent).unwrap();
	}
	fs::write(file, contents.as_ref())
		.map_err(|e| Error::other(std::format!("Failed to write {file:?}: {e}")))?;
	Ok(())
}

pub fn configure_cargo<'a>(
	command: &'a mut Command,
	config: &Config,
) -> Result<&'a mut Command, Error> {
	let sysroot = sysroot_dir()?;
	command
		.arg("--config")
		.arg(sysroot.join(if config.std { "std.toml" } else { "no_std.toml" }))
		// Can't set this variable via `config.toml`.
		.env("RUSTC_BOOTSTRAP", "1");
	Ok(command)
}

pub trait ConfigureCargo {
	fn configure_cargo_for_corevm(&mut self, config: &Config) -> Result<&mut Self, Error>;
}

impl ConfigureCargo for Command {
	fn configure_cargo_for_corevm(&mut self, config: &Config) -> Result<&mut Self, Error> {
		configure_cargo(self, config)
	}
}

#[derive(Debug, Default, Encode, Decode)]
pub struct Metadata {
	pub name: String,
	pub version: String,
	pub license: String,
	pub authors: Vec<String>,
}

#[derive(Default)]
pub struct Config {
	pub std: bool,
	pub polkavm: polkavm_linker::Config,
	pub metadata: Metadata,
}

pub fn program_from_elf(
	config: Config,
	elf: &[u8],
) -> Result<Vec<u8>, polkavm_linker::ProgramFromElfError> {
	let mut output = Vec::new();
	Compact::<u32>(config.metadata.encoded_size() as u32).encode_to(&mut output);
	config.metadata.encode_to(&mut output);
	output.append(&mut polkavm_linker::program_from_elf(config.polkavm, elf)?);
	Ok(output)
}

macro_rules! include_file {
	($path: expr) => {
		($path, include_bytes!(concat!(env!("OUT_DIR"), "/", $path)))
	};
}

macro_rules! stub_file {
	($path: expr) => {
		($path, &[])
	};
}

const SYSROOT: &[(&str, &[u8])] = &[
	include_file!("riscv64emac-corevm-linux-musl.json"),
	include_file!("lib/Scrt1.o"),
	include_file!("lib/crti.o"),
	include_file!("lib/crtn.o"),
	include_file!("lib/libc.a"),
	include_file!("lib/libclang_rt.builtins-riscv64.a"),
	stub_file!("lib/libunwind.a"),
];

const TARGET_FILE: &str = "riscv64emac-corevm-linux-musl.json";
pub const STD_TARGET_NAME: &str = "riscv64emac-corevm-linux-musl";
pub const NO_STD_TARGET_NAME: &str = "riscv64emac-unknown-none-polkavm";
