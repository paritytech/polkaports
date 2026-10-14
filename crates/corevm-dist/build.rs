use std::{
	fs,
	io::{BufWriter, Write},
	path::Path,
};

const B2SUM_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/b2sum.txt");
const DEFAULT_RELEASE_URL: &str = concat!(
	"https://github.com/paritytech/polkaports/releases/download/",
	env!("CARGO_PKG_VERSION"),
	"/"
);

fn main() {
	println!("cargo::rerun-if-env-changed=COREVM_DIST_RELEASE_URL");
	let release_url =
		std::env::var("COREVM_DIST_RELEASE_URL").unwrap_or_else(|_| DEFAULT_RELEASE_URL.into());
	let out_dir = std::env::var_os("OUT_DIR").unwrap();
	let data = fs::read_to_string(B2SUM_FILE).unwrap();
	let mut out_file =
		BufWriter::new(fs::File::create(Path::new(&out_dir).join("archives.rs")).unwrap());
	writeln!(&mut out_file, "const ARCHIVES: &[Archive] = &[").unwrap();
	for line in data.split('\n') {
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		let mut columns = line.split_whitespace();
		let hash = parse_hash(columns.next().unwrap());
		let filename = columns.next().unwrap();
		let mut fields = filename.strip_suffix(".tar.zst").unwrap().split('-');
		let name = fields.next().unwrap();
		let kernel = fields.next().unwrap();
		let arch = fields.next().unwrap();
		let archive =
			Archive { url: format!("{release_url}{filename}"), hash, filename, name, kernel, arch };
		writeln!(&mut out_file, "{archive:?},").unwrap();
	}
	writeln!(&mut out_file, "];").unwrap();
}

fn parse_hash(s: &str) -> [u8; 64] {
	let s = s.trim();
	let chars = s.as_bytes();
	if chars.len() != 128 {
		panic!("Invalid hash: {s:?}");
	}
	let mut hash = [0_u8; 64];
	for (i, byte) in hash.iter_mut().enumerate() {
		let byte_str = &s[2 * i..2 * (i + 1)];
		*byte = u8::from_str_radix(byte_str, 16).unwrap();
	}
	hash
}

#[derive(Debug)]
#[allow(unused)]
struct Archive<'a> {
	url: String,
	hash: [u8; 64],
	filename: &'a str,
	name: &'a str,
	kernel: &'a str,
	arch: &'a str,
}
