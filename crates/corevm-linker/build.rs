use std::{
	env::var_os,
	fs::{copy, create_dir},
	path::{Path, PathBuf},
	process::Command,
	sync::OnceLock,
};
use tempfile::TempDir;

fn main() {
	generate_target_file();
	build_musl();
}

fn generate_target_file() {
	let install_dir = PathBuf::from(var_os("OUT_DIR").unwrap());
	let target_file = Path::new(concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/../../sdk/riscv64emac-template-linux-musl.json"
	));
	let target = std::fs::read_to_string(target_file).unwrap();
	std::fs::write(
		install_dir.join("riscv64emac-corevm-linux-musl.json"),
		target.replace("@VENDOR@", "corevm"),
	)
	.unwrap();
}

fn build_musl() {
	let workdir = TempDir::new().unwrap();
	let build_dir = workdir.path().join("build");
	create_dir(&build_dir).unwrap();
	let install_dir = PathBuf::from(var_os("OUT_DIR").unwrap());
	let musl_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../libs/musl"))
		.canonicalize()
		.unwrap();
	let builtins_file = musl_dir.join("libclang_rt.builtins-riscv64.a");
	println!("cargo:rerun-if-env-changed=CC");
	println!("cargo:rerun-if-env-changed=AR");
	println!("cargo:rerun-if-env-changed=RANLIB");
	println!("cargo:rerun-if-env-changed=PATH");
	println!("cargo:rerun-if-changed={}", musl_dir.display());
	eprintln!("Musl dir: {musl_dir:?}");
	eprintln!("Build dir: {build_dir:?}");
	eprintln!("Install dir: {install_dir:?}");
	Command::new(musl_dir.join("configure"))
		.arg(format!("--prefix={}", install_dir.display()))
        .arg("--target=riscv64")
        .arg("--disable-shared")
        .env_clear()
		.env("PATH", var_os("PATH").unwrap_or_default())
		.env("CC", var_os("CC").unwrap_or_else(|| "clang".into()))
		.env("AR", var_os("AR").unwrap_or_else(|| "llvm-ar".into()))
		.env("RANLIB", var_os("RANLIB").unwrap_or_else(|| "llvm-ranlib".into()))
		.env("CFLAGS", "-Wno-shift-op-parentheses -Wno-unused-command-line-argument -fpic -fPIE -mrelax --target=riscv64-unknown-none-elf -march=rv64emac_zbb_xtheadcondmov -mabi=lp64e -ggdb")
		.env("LDFLAGS", "-Wl,--emit-relocs -Wl,--no-relax")
		.env("LIBCC", &builtins_file)
		.current_dir(&build_dir)
		.checked_status()
		.unwrap();
	Command::new("make")
		.with_job_server()
		.current_dir(&build_dir)
		.checked_status()
		.unwrap();
	Command::new("make")
		.with_job_server()
		.arg("install")
		.current_dir(&build_dir)
		.checked_status()
		.unwrap();
	copy(&builtins_file, install_dir.join("lib").join(builtins_file.file_name().unwrap())).unwrap();
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

trait JobServer {
	fn with_job_server(&mut self) -> &mut Self;
}

impl JobServer for Command {
	fn with_job_server(&mut self) -> &mut Self {
		if let Some(client) = get_job_server_client() {
			client.configure_make(self);
		}
		self
	}
}

fn get_job_server_client() -> Option<&'static jobserver::Client> {
	static CLIENT: OnceLock<Option<jobserver::Client>> = OnceLock::new();
	CLIENT.get_or_init(|| unsafe { jobserver::Client::from_env() }).as_ref()
}
