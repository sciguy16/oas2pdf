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

    pub fn out_file_name(&self) -> PathBuf {
        if let Some(out) = &self.out {
            out.clone()
        } else {
            let mut out = self.input.clone();
            if self.latex {
                out.set_extension("tex");
            } else {
                out.set_extension("pdf");
            }
            out
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn out_file_name() {
        let mut args = Args {
            out: None,
            latex: false,
            input: "/tmp/test.yaml".into(),
        };

        assert_eq!(args.out_file_name(), Path::new("/tmp/test.pdf"));
        args.latex = true;
        assert_eq!(args.out_file_name(), Path::new("/tmp/test.tex"));

        args.out = Some("/tmp/some-specified-name.docx".into());
        assert_eq!(
            args.out_file_name(),
            Path::new("/tmp/some-specified-name.docx")
        );
    }
}
