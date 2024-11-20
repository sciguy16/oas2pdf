use color_eyre::Result;
use openapiv3::{OpenAPI, Parameter, RefOr};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

mod args;
mod typst_world;

fn typst_escaper(input: &str) -> String {
    let mut out = String::new();
    for chr in input.chars() {
        if ['_'].contains(&chr) {
            out.push('\\');
        }
        out.push(chr);
    }
    out
}

const DEFAULT_TEMPLATE: &str = include_str!("../templates/output.typ");
const TEMPLATE_NAME: &str = "output";

type SectionName = String;
type Method = String;
type PathName = String;
type SectionEntry = BTreeMap<PathName, PathMethods>;
type PathMethods = BTreeMap<Method, PathInfo>;

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
struct Info {
    title: String,
    description: Option<String>,
    terms_of_service: Option<String>,
    version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
struct PathInfo {
    summary: Option<String>,
    description: Option<String>,
    operation_id: Option<String>,
    parameters: Vec<Parameter>,
}

#[allow(dead_code)]
enum HttpMethod {
    Get,
    Post,
    Delete,
    Patch,
    Put,
}

fn make_tera(template: &Option<PathBuf>) -> Result<tera::Tera> {
    let mut tera = tera::Tera::default();
    match template {
        Some(template) => {
            tera.add_template_file(template, Some(TEMPLATE_NAME))?
        }
        None => tera.add_raw_template(TEMPLATE_NAME, DEFAULT_TEMPLATE)?,
    }

    tera.autoescape_on(vec![""]);
    tera.set_escape_fn(typst_escaper);
    Ok(tera)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = args::Args::parse();

    if let Some(template) = args.save_template {
        std::fs::write(&template, DEFAULT_TEMPLATE.as_bytes())?;
        println!("Default template written to {}", template.display());
        return Ok(());
    }

    let input = std::fs::read_to_string(&args.input)?;
    let schema = serde_yaml::from_str::<OpenAPI>(&input)?;
    let transformed = transform_schema(&schema);
    dbg!(&transformed);

    let out_file_name = args.out_file_name();

    let tera = make_tera(&args.template)?;
    let tera_context = tera::Context::from_serialize(transformed)?;
    let rendered = tera.render(TEMPLATE_NAME, &tera_context)?;

    if args.typst {
        std::fs::write(&out_file_name, rendered)?;
        println!("Typst output written to `{}`", out_file_name.display());
        return Ok(());
    }

    let world =
        typst_world::SystemWorld::new(rendered.as_bytes().into(), rendered)?;
    let document = typst::compile(&world)
        .output
        .expect("Error compiling typst");
    let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .expect("Error exporting PDF");
    std::fs::write(&out_file_name, pdf)?;

    Ok(())
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
            let parameters = get
                .parameters
                .iter()
                .map(|param| {
                    let RefOr::Item(param) = param else { panic!() };
                    param.clone()
                })
                .collect();
            path_bit.insert(
                "get".to_string(),
                PathInfo {
                    summary: get.summary.clone(),
                    description: get.description.clone(),
                    operation_id: get.operation_id.clone(),
                    parameters,
                },
            );
        }
    }

    transformed
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
          parameters:
          - name: all
            description:  >-
                Show all containers. Only running containers are shown by default
                (i.e., this defaults to false)
            in: query
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
            ",
        )
        .unwrap();
        assert_eq!(transformed, expected);
    }
}
