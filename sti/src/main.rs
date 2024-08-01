use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    file: String,
    #[arg(short, long)]
    values: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let values: std::collections::HashMap<String, String> =
        serde_json::from_reader(std::fs::File::open(args.values)?).unwrap();
    let str = std::fs::read_to_string(args.file)?;

    let body = language::parse::parse(str).unwrap();
    let output = language::reduce::VerifiedTemplate::try_from_body_inputs(&body, &values)
        .unwrap()
        .reduce();
    println!("{output}");
    Ok(())
}
