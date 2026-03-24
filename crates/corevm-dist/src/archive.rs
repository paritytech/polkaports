include!(concat!(env!("OUT_DIR"), "/archives.rs"));

#[derive(Clone)]
pub struct Archive {
	pub url: &'static str,
	pub filename: &'static str,
	pub name: &'static str,
	pub kernel: &'static str,
	pub arch: &'static str,
	pub hash: [u8; 64],
}

impl Archive {
	fn filter_by_name(name: &str) -> Vec<Archive> {
		ARCHIVES.iter().filter(|archive| archive.name == name).cloned().collect()
	}

	pub fn all_tools() -> Vec<Archive> {
		Self::filter_by_name("tools")
	}

	pub fn find_tools() -> Option<Archive> {
		let kernel = Kernel::current()?.as_str();
		let arch = Arch::current()?.as_str();
		ARCHIVES
			.iter()
			.find(|archive| {
				archive.name == "tools" && archive.kernel == kernel && archive.arch == arch
			})
			.cloned()
	}

	pub fn sysroot() -> Archive {
		ARCHIVES
			.iter()
			.find(|archive| {
				archive.name == "sysroot" &&
					archive.kernel == "Linux" &&
					archive.arch == "riscv64emac"
			})
			.cloned()
			.expect("Sysroot is always available")
	}
}

#[derive(Clone, Copy)]
enum Kernel {
	Linux,
	Darwin,
}

impl Kernel {
	fn current() -> Option<Self> {
		if cfg!(target_os = "linux") {
			return Some(Self::Linux);
		}
		if cfg!(target_os = "macos") {
			return Some(Self::Darwin);
		}
		None
	}

	fn as_str(self) -> &'static str {
		match self {
			Self::Linux => "Linux",
			Self::Darwin => "Darwin",
		}
	}
}

#[derive(Clone, Copy)]
enum Arch {
	X86_64,
	Arm64,
}

impl Arch {
	fn current() -> Option<Self> {
		if cfg!(target_arch = "x86_64") {
			return Some(Self::X86_64);
		}
		if cfg!(target_arch = "aarch64") {
			return Some(Self::Arm64);
		}
		None
	}

	fn as_str(self) -> &'static str {
		match self {
			Self::X86_64 => "x86_64",
			Self::Arm64 => "arm64",
		}
	}
}
