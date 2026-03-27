#![doc = include_str!("../README.md")]
#![doc(hidden)]
use anyhow::anyhow;
use clap::Parser;

use std::path::PathBuf;

mod archive;
use self::archive::*;

mod command;

#[derive(clap::Parser)]
struct Args {
	#[clap(subcommand)]
	command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
	/// Install RISCV system root and build tools.
	///
	/// `$HOME/.corevm` by default.
	Install {
		/// Installation directory.
		#[clap(long = "prefix", value_name = "DIR", env = "COREVM_HOME")]
		prefix: Option<PathBuf>,
	},
	/// Update `corevm-dist` and install new system root and build tools.
	Update,
}

fn default_prefix() -> anyhow::Result<PathBuf> {
	let home_dir: PathBuf = std::env::var_os("HOME")
		.ok_or_else(|| anyhow!("Can't find home directory. Please set HOME environment variable."))?
		.into();
	Ok(home_dir.join(".corevm"))
}

pub enum PrefixKind {
	Default,
	Env,
	Other,
}

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	match args.command {
		Command::Install { prefix } => {
			let (prefix, prefix_kind) = match prefix {
				Some(prefix) => {
					let kind = if std::env::var_os("COREVM_HOME").is_some() {
						PrefixKind::Env
					} else {
						PrefixKind::Other
					};
					(prefix, kind)
				},
				None => (default_prefix()?, PrefixKind::Default),
			};
			command::install(&prefix, prefix_kind)?;
		},
		Command::Update => command::update()?,
	}
	Ok(())
}
