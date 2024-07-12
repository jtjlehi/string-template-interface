use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    files: Vec<String>,
    #[arg(short, long)]
    values: String,
}

fn main() {
    let args = Args::parse();
    println!("args: {args:?}");
}
