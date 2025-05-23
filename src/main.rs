#![expect(clippy::unnecessary_wraps)]

use color_eyre::Result;
use openapiv3::OpenAPI;
use std::{collections::HashMap, fmt::Write, path::PathBuf};
use tera::Value;

mod args;
mod transform_schema;
mod typst_world;

#[cfg(test)]
mod sample_tests;

const DEFAULT_TEMPLATE: &str = include_str!("../templates/output.typ");
const TEMPLATE_NAME: &str = "output";

fn typst_escaper(input: &str) -> String {
    let mut out = String::new();
    for chr in input.chars() {
        if ['_', '#', '$'].contains(&chr) {
            out.push('\\');
        }
        out.push(chr);
    }
    out
}

fn ref_or_is_ref(value: Option<&Value>, _args: &[Value]) -> tera::Result<bool> {
    if let Some(Value::Object(object)) = value {
        if object.contains_key("$ref") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ref_(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Value::Object(object) = value {
        if let Some(Value::String(value)) = object.get("$ref") {
            let ref_ =
                value.strip_prefix("#/components/schemas/").unwrap_or(value);

            return Ok(format!("#link(label(\"{ref_}\"), \"{ref_}\")").into());
        }
    }

    Ok(Value::default())
}

fn show_type(
    value: &Value,
    _args: &HashMap<String, Value>,
) -> tera::Result<Value> {
    let mut out = String::new();

    if let Some(Value::String(typ)) = value.get("type") {
        out.clone_from(typ);
    }

    if let Some(Value::Object(items)) = value.get("items") {
        let items = if let Some(Value::String(ref_)) = items.get("$ref") {
            let ref_ =
                ref_.strip_prefix("#/components/schemas/").unwrap_or(ref_);

            format!("#link(label(\"{ref_}\"), \"{ref_}\")")
        } else if let Some(Value::String(typ)) = items.get("type") {
            typ.clone()
        } else if let Some(Value::String(typ)) = items.get("format") {
            typ.clone()
        } else {
            String::new()
        };

        let _ = write!(&mut out, "[{items}]");
    }

    if out.is_empty() {
        Ok(Value::default())
    } else {
        Ok(out.into())
    }
}

fn make_tera(template: Option<&PathBuf>) -> Result<tera::Tera> {
    let mut tera = tera::Tera::default();
    match template {
        Some(template) => {
            tera.add_template_file(template, Some(TEMPLATE_NAME))?;
        }
        None => tera.add_raw_template(TEMPLATE_NAME, DEFAULT_TEMPLATE)?,
    }

    tera.autoescape_on(vec![""]);
    tera.set_escape_fn(typst_escaper);
    tera.register_tester("reference", ref_or_is_ref);
    tera.register_filter("ref", ref_);
    tera.register_filter("show_type", show_type);
    Ok(tera)
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = args::Args::parse();
    do_run(&args)
}

fn do_run(args: &args::Args) -> Result<()> {
    if let Some(template) = &args.save_template {
        std::fs::write(template, DEFAULT_TEMPLATE.as_bytes())?;
        println!("Default template written to {}", template.display());
        return Ok(());
    }

    let input = std::fs::read_to_string(&args.input)?;
    let schema = serde_yaml::from_str::<OpenAPI>(&input)?;
    let transformed = transform_schema::transform_schema(&schema);

    let out_file_name = args.out_file_name();

    let tera = make_tera(args.template.as_ref())?;
    let mut tera_context = tera::Context::from_serialize(transformed)?;
    tera_context.insert("param", &args.param());
    let rendered = tera.render(TEMPLATE_NAME, &tera_context)?;

    if args.typst {
        std::fs::write(&out_file_name, rendered)?;
        println!("Typst output written to `{}`", out_file_name.display());
        return Ok(());
    }

    let world = typst_world::SystemWorld::new(
        typst::foundations::Bytes::from_string(rendered.clone()),
        rendered,
    );
    let document = typst::compile(&world)
        .output
        .expect("Error compiling typst");
    let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .expect("Error exporting PDF");
    std::fs::write(&out_file_name, pdf)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::args::Args;

    const SCHEMA: &str = "
openapi: 3.0.0
info:
  title: Some API
  description: This is an API
  version: 1.0.0
paths:
  /some/path:
    get:
      summary: an interesting path
      description: a description
      operationId: aPath
      parameters:
        - name: query
          in: query
          description: a query parameter
          schema:
            type: integer
        - name: query-no-description
          in: query
          schema:
            type: integer
      responses:
        '200':
          description: ok
          content:
            application/json:
              schema:
                type: array
                items:
                  type: string
";

    pub fn rand_str() -> String {
        use rand::distr::{Alphanumeric, SampleString};
        Alphanumeric.sample_string(&mut rand::rng(), 20)
    }

    #[test]
    fn test_typst_escaper() {
        let cases = &[
            ("some text", None),
            ("Some\n fancy <i>text</i>", None),
            (
                "But this has an _underscore",
                Some("But this has an \\_underscore"),
            ),
        ];

        for &(input, expected) in cases {
            let escaped = typst_escaper(input);
            if let Some(expected) = expected {
                assert_eq!(escaped, expected);
            } else {
                assert_eq!(escaped, input);
            }
        }
    }

    #[test]
    fn save_default_template() {
        let template_path = format!("/tmp/default-template-{}", rand_str());
        let args = Args {
            out: None,
            typst: false,
            input: "".into(),
            save_template: Some((&template_path).into()),
            template: None,
            param: vec![],
        };

        do_run(&args).unwrap();

        let written = std::fs::read_to_string(&template_path).unwrap();

        assert_eq!(written, DEFAULT_TEMPLATE);
        std::fs::remove_file(template_path).unwrap();
    }

    #[test]
    fn typst_template() {
        let schema_path = format!("/tmp/schema-path-{}", rand_str());
        let typst_path = format!("{schema_path}.typ");
        let pdf_path = format!("{schema_path}.pdf");

        std::fs::write(&schema_path, SCHEMA.as_bytes()).unwrap();

        let mut args = Args {
            out: None,
            typst: true,
            input: (&schema_path).into(),
            save_template: None,
            template: None,
            param: vec![],
        };
        do_run(&args).unwrap();

        let written = std::fs::read_to_string(&typst_path).unwrap();
        insta::assert_snapshot!(written);

        // Try rendering the PDF. We can't test the PDF output as it's
        // nondeterministic (e.g. putting in today's date), but we can
        // catch typst's rendering errors
        args.typst = false;
        do_run(&args).unwrap();

        std::fs::remove_file(pdf_path).unwrap();
        std::fs::remove_file(typst_path).unwrap();
        std::fs::remove_file(schema_path).unwrap();
    }

    #[test]
    fn custom_template() {
        const TEMPLATE: &str = "
{{ info.title }}
#pagebreak()
== a section
{{ info.description }}
";

        let template_path = format!("/tmp/template-file-{}", rand_str());
        std::fs::write(&template_path, TEMPLATE.as_bytes()).unwrap();
        let schema_path = format!("/tmp/schema-file-{}", rand_str());
        std::fs::write(&schema_path, SCHEMA.as_bytes()).unwrap();

        let typst_path = format!("{schema_path}.typ");
        let pdf_path = format!("{schema_path}.pdf");

        let mut args = Args {
            out: None,
            typst: true,
            input: (&schema_path).into(),
            save_template: None,
            template: Some((&template_path).into()),
            param: vec![],
        };
        do_run(&args).unwrap();

        let written = std::fs::read_to_string(&typst_path).unwrap();
        insta::assert_snapshot!(written);

        // Try rendering the PDF. We can't test the PDF output as it's
        // nondeterministic (e.g. putting in today's date), but we can
        // catch typst's rendering errors
        args.typst = false;
        do_run(&args).unwrap();

        std::fs::remove_file(pdf_path).unwrap();
        std::fs::remove_file(typst_path).unwrap();
        std::fs::remove_file(schema_path).unwrap();
        std::fs::remove_file(template_path).unwrap();
    }
}
