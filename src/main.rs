use anyhow::Result;
use clap::{Arg, Command};

pub mod viz;

/// This function parses command line arguments and passes them
/// into the visualization workflow
///
/// Arguments:
/// --program-name (-p) program_name
fn main() -> Result<()> {
    let app = Command::new("anchor-viz")
        .version("0.1.0")
        .about("Visualize Anchor Programs")
        .author("@cavemanloverboy (Cavey Cool)");

    let program_name = Arg::new("program-name")
        .short('p')
        .takes_value(true)
        .help("name of anchor program. defaults to current dir name.");
    //.required(true);

    let app = app.arg(program_name);

    let matches = app.get_matches();

    // Extract program_name
    let name = matches
        .value_of("program-name")
        .map(|name| name.to_string());

    viz::visual(name)
}
