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

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	match args.command {
		Command::Install { prefix } => {
			let prefix = match prefix {
				Some(prefix) => prefix,
				None => default_prefix()?,
			};
			command::install(&prefix)?;
		},
		Command::Update => command::update()?,
	}
	Ok(())
}
