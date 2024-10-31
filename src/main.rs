use askama::Template;
use color_eyre::{eyre::eyre, Help, Result, SectionExt};
use indexmap::IndexMap;
use openapiv3::{OpenAPI, RefOr, Schema};
use std::collections::BTreeMap;

mod args;

mod filters {
    pub fn bracewrap(input: &str) -> askama::Result<String> {
        Ok(format!("{{{input}}}"))
    }

    pub fn brace_escape(input: &str) -> askama::Result<String> {
        Ok(input.replace('{', "\\{").replace('}', "\\}"))
    }
}

mod typ_escape {
    pub struct Typ;
    impl askama_escape::Escaper for Typ {
        fn write_escaped<W>(&self, mut wtr: W, input: &str) -> std::fmt::Result
        where
            W: std::fmt::Write,
        {
            for chr in input.chars() {
                if ['_'].contains(&chr) {
                    wtr.write_str("\\")?;
                }
                write!(wtr, "{chr}")?;
            }
            Ok(())
        }
    }
}

#[allow(dead_code)]
#[derive(Default)]
struct TransformedSchema<'schema> {
    schemas: BTreeMap<&'schema str, Schema>,
    sections: IndexMap<Option<&'schema String>, Vec<Method<'schema>>>,
}

#[allow(dead_code)]
struct Method<'method> {
    method: HttpMethod,
    path: &'method String,
    description: Option<&'method String>,
    parameters: Vec<String>,
}

#[allow(dead_code)]
enum HttpMethod {
    Get,
    Post,
    Delete,
    Patch,
    Put,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = args::Args::parse();

    if !args.latex {
        check_typst()?;
    }

    let input = std::fs::read_to_string(&args.input)?;
    let schema = serde_yaml::from_str::<OpenAPI>(&input)?;
    let _transformed = transform_schema(&schema);

    let out_file_name = args.out_file_name();

    let templ = DocsTemplate { schema: &schema };
    if args.latex {
        let mut out_file = std::fs::File::create(&out_file_name)?;
        templ.write_into(&mut out_file)?;
        println!("LaTeX output written to `{}`", out_file_name.display());
        return Ok(());
    }

    let mut typst_file = tempfile::Builder::new()
        .prefix("oas2pdf")
        .suffix(".typ")
        .tempfile()?;
    templ.write_into(&mut typst_file)?;

    let status = std::process::Command::new("typst")
        .args(["compile", "--format", "pdf"])
        .args([typst_file.path(), &out_file_name])
        .output()?;
    if !status.status.success() {
        let typst_file_path = typst_file.path().display().to_string();
        typst_file.keep()?;
        return Err(eyre!("typst failed")
            .with_section(|| eyre!("{typst_file_path}",).header("Typst file:"))
            .with_section(move || {
                eyre!("{}", String::from_utf8_lossy(&status.stdout))
                    .header("stdout:")
            })
            .with_section(move || {
                eyre!("{}", String::from_utf8_lossy(&status.stderr))
                    .header("stderr:")
            }));
    }

    Ok(())
}

fn check_typst() -> Result<()> {
    let status = std::process::Command::new("typst")
        .status()
        .with_section(|| "typst not found")?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!("typst not found in $PATH"))
    }
}

fn transform_schema(schema: &OpenAPI) -> TransformedSchema {
    let mut transformed = TransformedSchema::default();

    for (path_name, path_item) in &schema.paths.paths {
        let RefOr::Item(path_item) = path_item else {
            panic!("References not supported in path items");
        };
        if let Some(get) = &path_item.get {
            let tag = get.tags.first();
            transformed.sections.entry(tag).or_default().push(Method {
                method: HttpMethod::Get,
                path: path_name,
                description: get.description.as_ref(),
                parameters: Vec::new(),
            });
        }
    }

    transformed
}

#[derive(Template)]
#[template(path = "output.typ")]
struct DocsTemplate<'schema> {
    schema: &'schema OpenAPI,
}
