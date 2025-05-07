use clap::Parser;
use std::{collections::HashMap, path::PathBuf};

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
    #[clap(
        long,
        value_parser = parse_key_val::<String, String>,
        help = "Key=value pairs to pass to the template. Reference as e.g. \
            `param.org`"
    )]
    pub param: Vec<(String, String)>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
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

    pub fn param(&self) -> HashMap<String, String> {
        self.param.iter().cloned().collect()
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
            param: vec![],
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
