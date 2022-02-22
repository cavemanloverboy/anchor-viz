use anyhow::Result;
use clap::{Arg, Command};

pub mod viz;

fn main() -> Result<()> {
    let app = Command::new("anchor-viz")
        .version("0.1.0")
        .about("Visualize Anchor Programs")
        .author("@cavemanloverboy (Cavey Cool)");

    let name_option = Arg::new("program-name")
        .short('p')
        .takes_value(true)
        .help("name of anchor program. defaults to current dir name.");
    //.required(true);

    let app = app.arg(name_option);

    let matches = app.get_matches();

    // Extract the program name
    let name = match matches.value_of("program-name") {
        Some(name) => Some(name.to_string()),
        None => None,
    };

    viz::visual(name)
}
