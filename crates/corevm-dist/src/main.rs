use clap::Parser;

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
	Install,
}

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	match args.command {
		Command::Install => command::install(),
	}
}
