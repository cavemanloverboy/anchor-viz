use anyhow::Result;

pub mod viz;

/// This function parses command line arguments and passes them
/// into the visualization workflow
///
/// Arguments:
/// --program-name (-p) program_name
fn main() -> Result<()> {
    // Parse args
    let args = Args::parse();

    viz::visual(args.program_name, args.width)
}

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of programto visualize
    #[clap(short, long)]
    program_name: Option<String>,

    /// Number of accounts, arguments per instruction column
    #[clap(short, long, default_value_t = 2)]
    width: usize,
}

#[test]
fn test_0() {
    viz::visual(Some("test_0/programs/test_0".to_string()), 2).unwrap();
}

#[test]
fn test_1() {
    viz::visual(Some("test_1/programs/test_1".to_string()), 2).unwrap();
}

