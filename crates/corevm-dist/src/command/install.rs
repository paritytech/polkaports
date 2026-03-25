use anyhow::anyhow;
use memmap2::Mmap;
use tempfile::TempDir;

use std::{ffi::OsString, path::Path};

use crate::Archive;

const ENV_SH: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/env.sh"));

pub fn install(prefix: &Path) -> anyhow::Result<()> {
	let tmpdir = TempDir::new()?;
	// System root.
	let sysroot_archive = Archive::sysroot();
	download_and_unpack(&sysroot_archive, &prefix.join("sysroot"), tmpdir.path())?;
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
	download_and_unpack(&tools_archive, &prefix.join("bin"), tmpdir.path())?;
	write_env_file(prefix)?;
	println!(
        "\nDone!\n\n\
        Type the following command to activate the toolchain.\n\n    . \"$HOME\"/.corevm-dist/env\n\n\
        NOTE: Toolchain requires LLVM v20 (clang, clang++, lld) to work."
    );
	Ok(())
}

fn download_and_unpack(archive: &Archive, output_dir: &Path, tmpdir: &Path) -> anyhow::Result<()> {
	let archive_file = tmpdir.join(archive.filename);
	download_file(archive.url, &archive_file)?;
	unpack_archive(&archive_file, &archive.hash, output_dir)?;
	fs::remove_file(&archive_file)?;
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
		.args(["--fail", "--location", "--silent", "--output"])
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
) -> anyhow::Result<()> {
	eprintln!("📦 Unpacking {archive_file:?} to {output_dir:?}...");
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
