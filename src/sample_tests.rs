use crate::{args::Args, do_run, test::rand_str};
use std::path::Path;

fn run_sample_test<P: AsRef<Path>>(input: P) -> String {
    let typst_path = format!("/tmp/sample-typst-out-{}", rand_str());
    let mut args = Args {
        out: Some((&typst_path).into()),
        typst: true,
        input: input.as_ref().into(),
        save_template: None,
        template: None,
    };
    do_run(&args).unwrap();
    // Store the typst file for snapshot testing
    let rendered = std::fs::read_to_string(&typst_path).unwrap();

    // try rendering the PDF to check for syntax errors
    args.typst = false;
    do_run(&args).unwrap();

    std::fs::remove_file(typst_path).unwrap();
    rendered
}

#[test]
fn docker_api() {
    insta::assert_snapshot!(run_sample_test("samples/docker.yaml"));
}
