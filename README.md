# oas2pdf
Create PDF documentation from OpenAPI schemas!

## Usage
Install with `cargo install --locked --git https://github.com/sciguy16/oas2pdf`,
then run `oas2pdf --help`.

To convert a YAML schema to PDF, run e.g. `oas2pdf schema.yaml` or
`oas2pdf schema.yaml --out some-other-name.pdf`.

## Limitations
* This has not been tested with a wide variety of schema files - if you have
  a schema that doesn't work with this tool then please file a PR to add it to
  the samples directory
* This currently works by templating fields from the schema directly into a
  [Typst](https://typst.app) template. As such, it is possible for malicious
  schemas to read arbitrary files from the filsystem. I don't think there's a
  mechanism for writing files or executing shell commands, but haven't fully
  confirmed this. As ever, it is inadvisable to run this on schema files from
  untrusted sources

## Licence
This project is licenced  under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

   at your option.
