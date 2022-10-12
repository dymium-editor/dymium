//! Program to check that `capdata.yaml` (or any other `.yaml`) is valid
//!
//! `verify-caps` also prints a brief summary of the `$TERM` mappings (i.e., which terminals are
//! associated with each value of `$TERM`).

use std::path::PathBuf;
use std::process::exit;

use dymium_term::capinfo::{self, TerminalName};

static USAGE: &str = concat!("Usage: verify-caps ( -h | <FILE> )",);

struct Args {
    file: PathBuf,
}

fn main() -> Result<(), capinfo::LoadTermCapsError> {
    let args = Args::parse();
    let grouped_caps = capinfo::TermCapSet::load_all_from_file(&args.file)?.group_by_env_var();

    for v in grouped_caps.env_vars() {
        println!("{v}:");
        let g = grouped_caps.get(v).unwrap();
        for TerminalName { compact, pretty, .. } in g.members() {
            println!(" - {compact} ({pretty:?})");
        }
    }

    Ok(())
}

impl Args {
    fn parse() -> Self {
        let mut args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() != 1 {
            eprintln!("{USAGE}");
            exit(1);
        } else if args[0] == "-h" {
            println!("{USAGE}");
            exit(0);
        }

        Args { file: PathBuf::from(args.remove(0)) }
    }
}
