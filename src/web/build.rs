extern crate web_build;
extern crate codevisual_static_web;

use std::path::Path;
use std::io::Write;

fn main() {
    let codevisual_dts = Path::new("src").join("js").join("typings").join("codevisual.d.ts");
    std::fs::File::create(&codevisual_dts).unwrap()
        .write_all(codevisual_static_web::DTS.as_ref()).unwrap();
    web_build::compile_ts(&Path::new("src").join("js"), &Path::new("lib.js"));
    std::fs::remove_file(&codevisual_dts).unwrap();
}
