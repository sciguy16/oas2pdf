use askama::Template;
use color_eyre::{eyre::eyre, Help, Result, SectionExt};
use openapiv3::{OpenAPI, RefOr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod args;

// mod filters {
//     pub fn bracewrap(input: &str) -> askama::Result<String> {
//         Ok(format!("{{{input}}}"))
//     }

//     pub fn brace_escape(input: &str) -> askama::Result<String> {
//         Ok(input.replace('{', "\\{").replace('}', "\\}"))
//     }
// }

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

type SectionName = String;
type Method = String;
type PathName = String;
type SectionEntry = BTreeMap<PathName, PathMethods>;
type PathMethods = BTreeMap<Method, PathInfo>;

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
struct TransformedSchema {
    info: Info,
    sections: BTreeMap<SectionName, SectionEntry>,
}

/*
info:
    title:
    description:
    termsOfService:
    version:
sections:
    section_name:
        - /path:
            get:
                summary:
                description:
                operation_id:
                parameters: []
*/

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
struct Info {
    title: String,
    description: Option<String>,
    terms_of_service: Option<String>,
    version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
struct PathInfo {
    summary: Option<String>,
    description: Option<String>,
    operation_id: Option<String>,
    parameters: Vec<()>,
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

    transformed.info.title = schema.info.title.clone();
    transformed.info.description = schema.info.description.clone();
    transformed.info.terms_of_service = schema.info.terms_of_service.clone();
    transformed.info.version = schema.info.version.clone();

    for (path_name, path_item) in &schema.paths.paths {
        let RefOr::Item(path_item) = path_item else {
            panic!("References not supported in path items");
        };

        if let Some(get) = &path_item.get {
            let tag =
                get.tags.first().cloned().unwrap_or_else(|| "Other".into());
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            path_bit.insert(
                "get".to_string(),
                PathInfo {
                    summary: get.summary.clone(),
                    description: get.description.clone(),
                    operation_id: get.operation_id.clone(),
                    parameters: vec![],
                },
            );
        }
    }

    transformed
}

#[derive(Template)]
#[template(path = "output.typ")]
struct DocsTemplate<'schema> {
    schema: &'schema OpenAPI,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    const SAMPLE:&str = "openapi: 3.0.0
info:
  title: Docker Remote API
  description: The API for each docker installation.
  termsOfService: 'http://example.com/tos/'
  version: v1.21
paths:
  /containers/json:
    get:
      summary: List containers
      description: List containers
      operationId: findAllContainers
      parameters:
        - name: all
          in: query
          description: >-
            Show all containers. Only running containers are shown by default
            (i.e., this defaults to false)
          schema:
            type: boolean
            default: false
        - name: limit
          in: query
          description: 'Show  last created containers, include non-running ones.'
          schema:
            type: integer
        - name: since
          in: query
          description: 'Show only containers created since Id, include non-running ones.'
          schema:
            type: string
        - name: before
          in: query
          description: 'Show only containers created before Id, include non-running ones.'
          schema:
            type: string
        - name: size
          in: query
          description: '1/True/true or 0/False/false, Show the containers sizes.'
          schema:
            type: boolean
        - name: filters
          in: query
          description: >-
            A JSON encoded value of the filters (a map[string][]string) to
            process on the containers list
          schema:
            type: array
            items:
              type: string
      responses:
        '200':
          description: no error
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/ContainerConfig'
            text/plain:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/ContainerConfig'
        '400':
          description: bad parameter
        '500':
          description: server error
      tags:
        - Container
";

    #[test]
    fn test_transform() {
        let schema = serde_yaml::from_str(SAMPLE).unwrap();
        let transformed = transform_schema(&schema);
        let expected = serde_yaml::from_str::<TransformedSchema>(
            "
info:
  title: Docker Remote API
  description: The API for each docker installation.
  terms_of_service: 'http://example.com/tos/'
  version: v1.21
sections:
  Container:
    /containers/json:
        get:
          summary: List containers
          description: List containers
          operation_id: findAllContainers
          parameters: []
            ",
        )
        .unwrap();
        assert_eq!(transformed, expected);
    }
}
