#!/usr/bin/env -S cargo +nightly -Zscript
```cargo
[dependencies]
clap = { version = "4.4", features = ["derive"] }
toml = { version = "0.8" }
```

use clap::Parser;
use std::process::ExitCode;
use std::fs::read_to_string;
use std::error::Error;
use toml::Table;
use toml::from_str;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
	#[arg(short = 'u')]
	update: bool,

	#[arg(short = 'g')]
	generate: bool,
}

fn main() -> ExitCode {
	let args = Args::parse();

	return match (args.update, args.generate) {
		(true, true) => {
			eprintln!("error: can specify only one of --update and --generate");
			ExitCode::FAILURE
		}
		(false, false) => {
			eprintln!("error: must specify one of --update or --generate");
			ExitCode::FAILURE
		}
		(true, false) => if let Err(err) = update() {
			eprintln!("error: {}", err);
			ExitCode::FAILURE
		} else {
			ExitCode::SUCCESS
		},
		(false, true) => if let Err(err) = generate() {
			eprintln!("error: {}", err);
			ExitCode::FAILURE
		} else {
			ExitCode::SUCCESS
		},
	}

}

/// Update the cached metadata.
fn update() -> Result<(), Box<dyn Error>> {
	/*
	The intent of this function is that it will read
	the `languages.toml` file, then filter down to the
	ones that are on GitHub and update the metadata
	file, found at `.data/stars.toml` with star data
	based on the GitHub API responses, same deal with
	`.data/active.toml`.
	*/

	let langs = toml_from_file("languages.toml")?;

	Ok(())
}

/// Regenerate the README.
fn generate() -> Result<(), Box<dyn Error>> {
	/*
	This takes the list of languages from
	`languages.toml`, and also loads `.data/stars.toml`,
	and uses the data from both to generate a new
	`README.md` file.
	*/

	let langs = toml_from_file("languages.toml")?;

	Ok(())
}

/// Read the TOML data from a file.
fn toml_from_file(file: &str) -> Result<Table, Box<dyn Error>> {
	Ok(from_str(&read_to_string(file)?)?)
}


