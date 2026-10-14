use anstyle::{AnsiColor, Style};
use anyhow::anyhow;
use memmap2::Mmap;

use std::{
	ffi::OsString,
	io::IsTerminal as _,
	path::{Path, PathBuf},
};

use crate::{Archive, PrefixKind};

const ENV_SH: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/env.sh"));

pub fn install(prefix: &Path, prefix_kind: PrefixKind) -> anyhow::Result<()> {
	let download_dir = download_dir();
	// System root.
	let sysroot_archive = Archive::sysroot();
	download_and_unpack(&sysroot_archive, &prefix.join("sysroot"), &download_dir)?;
	// Tools.
	let tools_archive = Archive::find_tools().ok_or_else(|| {
		use std::fmt::Write;
		let mut available = String::new();
		for archive in Archive::all_tools() {
			let _ = writeln!(&mut available, "{}", archive.filename);
		}
		anyhow!(
			"Failed to find compatible binary distribution archives. \
            Currently available archives:\n{available}"
		)
	})?;
	download_and_unpack(&tools_archive, &prefix.join("bin"), &download_dir)?;
	write_env_file(prefix)?;
	print_instructions(prefix, prefix_kind);
	Ok(())
}

fn download_dir() -> PathBuf {
	std::env::var_os("XDG_CACHE_HOME")
		.map(PathBuf::from)
		.or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
		.unwrap_or_else(std::env::temp_dir)
}

fn print_instructions(prefix: &Path, prefix_kind: PrefixKind) {
	let in_shell_prefix = match prefix_kind {
		PrefixKind::Default => "\"$HOME\"/.corevm/env".to_string(),
		PrefixKind::Env => "\"$COREVM_HOME\"/env".to_string(),
		PrefixKind::Other => format!("{:?}", prefix.join("env")),
	};
	let warn = if std::io::stdout().is_terminal() {
		Style::new().bold().fg_color(Some(AnsiColor::Yellow.into()))
	} else {
		Style::new()
	};
	println!(
		"\nDone!\n\n\
        Type the following command to activate the toolchain.\n\n    . {in_shell_prefix}\n\n\
        {warn}NOTE{warn:#}: Toolchain requires LLVM v20 (clang, clang++, lld) to work.",
	);
}

fn download_and_unpack(
	archive: &Archive,
	output_dir: &Path,
	download_dir: &Path,
) -> anyhow::Result<()> {
	let archive_file = download_dir.join(archive.filename);
	let mut unpacked = false;
	if archive_file.exists() {
		unpacked = unpack_archive(&archive_file, &archive.hash, output_dir, false).is_ok();
	}
	if !unpacked {
		fs::create_dir_all(download_dir)?;
		download_file(archive.url, &archive_file)?;
	}
	unpack_archive(&archive_file, &archive.hash, output_dir, true)?;
	Ok(())
}

fn write_env_file(prefix: &Path) -> anyhow::Result<()> {
	let env_file = prefix.join("env");
	let env_file_tmp = prefix.join(".env.tmp");
	eprintln!("📦 Writing {env_file:?}...");
	fs::remove_file(&env_file_tmp).ok_if_not_found()?;
	fs::write(&env_file_tmp, ENV_SH)?;
	fs::remove_file(&env_file).ok_if_not_found()?;
	fs::rename(&env_file_tmp, &env_file)?;
	Ok(())
}

#[cfg(feature = "reqwest")]
fn download_file(url: &str, output_file: &Path) -> anyhow::Result<()> {
	eprintln!("📥 Downloading {url}...");
	let mut response = reqwest::blocking::get(url)?.error_for_status()?;
	let mut file = fs::File::create(output_file)?;
	response.copy_to(&mut file)?;
	Ok(())
}

#[cfg(not(feature = "reqwest"))]
fn download_file(url: &str, output_file: &Path) -> anyhow::Result<()> {
	eprintln!("📥 Downloading {url}...");
	let status = std::process::Command::new("curl")
		.stdin(std::process::Stdio::null())
		.args(["--fail", "--location", "--output"])
		.arg(output_file)
		.arg(url)
		.status()
		.map_err(|e| anyhow!("Failed to execute `curl`: {e}"))?;
	if !status.success() {
		return Err(anyhow!("Failed to execute `curl`: non-zero exit code"));
	}
	Ok(())
}

fn unpack_archive(
	archive_file: &Path,
	expected_hash: &[u8; 64],
	output_dir: &Path,
	verbose: bool,
) -> anyhow::Result<()> {
	if verbose {
		eprintln!("🔍 Verifying {archive_file:?}...");
	}
	let input_file = fs::File::open(archive_file)?;
	let archive_data = unsafe { Mmap::map(&input_file)? };
	let expected_hash = blake2b_simd::Hash::from(expected_hash);
	let mut hasher = blake2b_simd::Params::new().hash_length(64).to_state();
	hasher.update(&archive_data);
	let actual_hash = hasher.finalize();
	if actual_hash != expected_hash {
		return Err(anyhow!(
			"Failed to unpack {archive_file:?}: hash mismatch\n\
            expected hash {}\n\
            actual hash   {}",
			expected_hash.to_hex(),
			actual_hash.to_hex(),
		))
	}
	if verbose {
		eprintln!("📦 Unpacking {archive_file:?} to {output_dir:?}...");
	}
	let mut archive = tar::Archive::new(zstd::Decoder::new(&archive_data[..])?);
	let output_dir_tmp = output_dir.parent().expect("Output directory has parent").join({
		let mut name = OsString::new();
		name.push(".");
		name.push(output_dir.file_name().expect("Output directory has name"));
		name.push(".tmp");
		name
	});
	fs::remove_dir_all(&output_dir_tmp).ok_if_not_found()?;
	fs::create_dir_all(&output_dir_tmp)?;
	archive.unpack(&output_dir_tmp)?;
	fs::remove_dir_all(output_dir).ok_if_not_found()?;
	fs::rename(&output_dir_tmp, output_dir)?;
	Ok(())
}

trait OkIfNotFound {
	fn ok_if_not_found(self) -> Self;
}

impl OkIfNotFound for std::io::Result<()> {
	fn ok_if_not_found(self) -> Self {
		match self {
			Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
			other => other,
		}
	}
}
