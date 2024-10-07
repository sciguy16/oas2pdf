use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    #[clap(
        long,
        help = "Output file. If not specified then same as input \
        	but ending with .pdf"
    )]
    pub out: Option<PathBuf>,
    #[clap(long, help = "Output just the generated LaTeX file")]
    pub latex: bool,
    #[clap(help = "Input file")]
    pub input: PathBuf,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
