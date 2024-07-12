use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Parser, Debug)]
struct Args {
    files: Vec<String>,
    #[arg(short, long)]
    values: String,
}

#[derive(Deserialize, Debug)]
struct Values(HashMap<String, String>);

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let values: Values = serde_json::from_reader(std::fs::File::open(args.values)?).unwrap();
    println!("values: {values:?}");
    Ok(())
}
