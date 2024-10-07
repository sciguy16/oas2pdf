use color_eyre::{eyre::eyre, Help, Result};
use indexmap::IndexMap;
use openapiv3::{OpenAPI, PathItem, RefOr, Schema};
use std::collections::BTreeMap;

mod args;

#[derive(Default)]
struct TransformedSchema<'schema> {
    schemas: BTreeMap<&'schema str, Schema>,
    sections: IndexMap<Option<&'schema String>, Vec<Method<'schema>>>,
}

struct Method<'method> {
    method: HttpMethod,
    path: &'method String,
    description: Option<&'method String>,
    parameters: Vec<String>,
}

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
        check_pdflatex()?;
    }

    let input = std::fs::read_to_string(&args.input)?;
    let schema = serde_yaml::from_str::<OpenAPI>(&input)?;
    let transformed = transform_schema(&schema);

    Ok(())
}

fn check_pdflatex() -> Result<()> {
    let status = std::process::Command::new("pdflatex")
        .arg("-v")
        .status()
        .with_section(|| "pdflatex not found")?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!("pdflatex not found in $PATH"))
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
