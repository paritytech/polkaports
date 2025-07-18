use std::{
	env::{var, var_os},
	fs::{copy, create_dir, create_dir_all},
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
	let workdir = workdir.path();
	let build_dir = workdir.join("build");
	create_dir(&build_dir).unwrap();
	let install_dir = PathBuf::from(var_os("OUT_DIR").unwrap());
	let musl_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../libs/musl"))
		.canonicalize()
		.unwrap();
	let builtins_file = musl_dir.join("libclang_rt.builtins-riscv64.a");
	let picoalloc_dir = musl_dir.join("src").join("malloc").join("picoalloc");
	println!("cargo:rerun-if-env-changed=CC");
	println!("cargo:rerun-if-env-changed=AR");
	println!("cargo:rerun-if-env-changed=RANLIB");
	println!("cargo:rerun-if-env-changed=PATH");
	println!("cargo:rerun-if-changed={}", musl_dir.display());
	eprintln!("Musl dir: {musl_dir:?}");
	eprintln!("Build dir: {build_dir:?}");
	eprintln!("Install dir: {install_dir:?}");
	create_dir_all(&picoalloc_dir).unwrap();
	Command::new(musl_dir.join("configure"))
        .arg(format!("--prefix={}", install_dir.display()))
        .arg("--with-malloc=picoalloc")
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
	build_picoalloc(workdir, &musl_dir);
	Command::new("make")
		.with_job_server()
		.arg("install")
		.current_dir(&build_dir)
		.checked_status()
		.unwrap();
	copy(&builtins_file, install_dir.join("lib").join(builtins_file.file_name().unwrap())).unwrap();
}

fn build_picoalloc(workdir: &Path, musl_dir: &Path) {
	let is_clippy = match var("RUSTC_WORKSPACE_WRAPPER") {
		Ok(wrapper) => wrapper.ends_with("clippy-driver"),
		Err(_) => false,
	};
	if is_clippy {
		// Clippy fails on picoalloc build.
		return;
	}
	let build_dir = workdir.join("picoalloc");
	let archive_dir = workdir.join("archive");
	let release_dir = build_dir
		.join("target")
		.join("riscv64emac-unknown-none-polkavm")
		.join("release");
	create_dir_all(&archive_dir).unwrap();
	Command::new("git")
		.args(["clone", "--depth=1", "--branch", PICOALLOC_TAG, "--quiet", PICOALLOC_URL])
		.arg(&build_dir)
		.checked_status()
		.unwrap();
	Command::new(var_os("CARGO").unwrap_or_else(|| "cargo".into()))
		.with_job_server()
		.current_dir(&build_dir)
		.env("RUSTC_BOOTSTRAP", "1")
		.args([
			"build",
			"-Zbuild-std=core,alloc",
			"--quiet",
			"--package",
			"picoalloc_native",
			"--release",
			"--features",
			"corevm",
		])
		.arg("--target")
		.arg(polkavm_linker::target_json_64_path().unwrap())
		.checked_status()
		.unwrap();
	Command::new("find")
		.current_dir(build_dir.join("target"))
		.checked_status()
		.unwrap();
	let ar = var_os("AR").unwrap_or_else(|| "llvm-ar".into());
	let lib_dir = musl_dir.join("lib");
	Command::new(&ar)
		.current_dir(&lib_dir)
		.arg("x")
		.arg(release_dir.join("libpicoalloc_native.a"))
		.checked_status()
		.unwrap();
	let mut objects = Vec::new();
	for entry in std::fs::read_dir(&lib_dir).unwrap() {
		let entry = entry.unwrap();
		let file_name = entry.file_name();
		let Some(file_name_str) = file_name.to_str() else {
			continue;
		};
		if file_name_str.starts_with("picoalloc") && file_name_str.ends_with(".o") {
			objects.push(file_name);
			eprintln!("{:?}", entry.path());
		}
	}
	Command::new(&ar)
		.current_dir(&lib_dir)
		.arg("r")
		.arg("libc.a")
		.args(objects)
		.checked_status()
		.unwrap();
}

const PICOALLOC_TAG: &str = "v5.2.0";
const PICOALLOC_URL: &str = "https://github.com/koute/picoalloc";

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
