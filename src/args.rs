use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    #[clap(
        long,
        help = "Output file. If not specified then same as input \
        	but ending with .pdf or .typ"
    )]
    pub out: Option<PathBuf>,
    #[clap(long, help = "Output just the generated Typst file")]
    pub typst: bool,
    #[clap(help = "Input file")]
    pub input: PathBuf,
    #[clap(long, help = "Save the default template to a file")]
    pub save_template: Option<PathBuf>,
    #[clap(long, help = "Use the provided template instead of the default")]
    pub template: Option<PathBuf>,
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
            if self.typst {
                out.set_extension("typ");
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
            typst: false,
            input: "/tmp/test.yaml".into(),
            save_template: None,
            template: None,
        };

        assert_eq!(args.out_file_name(), Path::new("/tmp/test.pdf"));
        args.typst = true;
        assert_eq!(args.out_file_name(), Path::new("/tmp/test.typ"));

        args.out = Some("/tmp/some-specified-name.docx".into());
        assert_eq!(
            args.out_file_name(),
            Path::new("/tmp/some-specified-name.docx")
        );
    }
}
