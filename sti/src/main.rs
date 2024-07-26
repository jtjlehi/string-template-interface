use clap::Parser;
use language::Inputs;

#[derive(Parser, Debug)]
struct Args {
    files: Vec<String>,
    #[arg(short, long)]
    values: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("values: {args:?}");
    Ok(())
}
