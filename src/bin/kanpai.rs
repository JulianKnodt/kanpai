use clap::Parser;
use std::fs::File;
use std::io::Read;

fn main() {
    let args = Args::parse();
    let mut f = File::open(args.file).expect("File does not exist");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Failed to read input file");

    let prog_parser = kanpai::ProgramParser::new();

    prog_parser.parse(&s).expect("Failed to parse input file");
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    file: String,
}
