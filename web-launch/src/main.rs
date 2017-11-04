extern crate prelude;
extern crate codevisual_static_web;
extern crate open;
extern crate fs_extra;
extern crate argparse;

#[allow(unused_imports)]
use prelude::*;

use std::path::Path;
use std::process::Command;
use std::io::Write;
use std::fs::{self, File};

fn main() {
    std::env::set_current_dir("..").unwrap();

    let mut release = true;
    let mut target = String::from("asmjs");
    let mut open = false;
    let mut sync = false;
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.refer(&mut release)
            .add_option(&["--debug"], argparse::StoreFalse, "Build with optimizations disabled");
        ap.refer(&mut target)
            .add_option(&["--target"], argparse::Store, "asmjs | wasm32");
        ap.refer(&mut open)
            .add_option(&["--open"], argparse::StoreTrue, "Open browser after build");
        ap.refer(&mut sync)
            .add_option(&["--sync"], argparse::StoreTrue, "Sync to remote host");
        ap.parse_args_or_exit();
    }
    build(release, &target);

    copy_resources(release, &target);

    if sync {
        let mut command = Command::new("rsync");
        command.arg("-avz").arg("--delete");
        command.arg(format!("target/web/{}/troll-invasion/*", target));
        command.arg(format!("pi@pi.kuviman.com:/home/pi/codevisual/{}/troll-invasion", target));
        assert!(command.status().unwrap().success());
    }

    if open {
        let port = 8123; // thread_rng().gen_range(8000, 9000);

        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            assert!(open::that(format!("http://127.0.0.1:{}/index.html", port)).unwrap().success());
        });

        let mut server = Command::new("python");
        server.current_dir(Path::new("target").join("web").join(&target).join("troll-invasion"));
        server.arg("-m").arg("SimpleHTTPServer");
        server.arg(port.to_string());
        assert!(server.status().unwrap().success());
    }
}

fn build<T: AsRef<Path>>(release: bool, target: T) {
    // TODO: fixed memory should be better
    std::env::set_var("EMMAKEN_CFLAGS", "-s ALLOW_MEMORY_GROWTH=1");
    let cargo_target = format!("{}-unknown-emscripten", target.as_ref().to_str().unwrap());
    let mut command = Command::new("cargo");
    command.arg("build");
    if release {
        command.arg("--release");
    }
    command.arg(format!("--target={}", cargo_target));
    assert!(command.status().unwrap().success());
}

fn copy_resources<T: AsRef<Path>>(release: bool, target: T) {
    let cargo_config = if release { "release" } else { "debug" };
    let cargo_target = format!("{}-unknown-emscripten", target.as_ref().to_str().unwrap());
    let build_dir = Path::new("target").join(cargo_target).join(cargo_config);
    let target_dir = Path::new("target").join("web").join(&target).join("troll-invasion");
    fs::create_dir_all(&target_dir).unwrap();
    File::create(target_dir.join("codevisual.html")).unwrap()
        .write_all(codevisual_static_web::HTML.as_ref()).unwrap();
    File::create(target_dir.join("codevisual.css")).unwrap()
        .write_all(codevisual_static_web::CSS.as_ref()).unwrap();
    File::create(target_dir.join("codevisual.js")).unwrap()
        .write_all(codevisual_static_web::JS.as_ref()).unwrap();
    fn copy_dir_contents<P, Q>(source: P, target: Q)
        where P: AsRef<Path>, Q: AsRef<Path> {
        let entries = fs::read_dir(source).unwrap().map(|entry| entry.unwrap().path()).collect();
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        fs_extra::copy_items(&entries, target, &options).unwrap();
    }
    copy_dir_contents(Path::new("codevisual").join("static_web").join("static"), &target_dir);
    copy_dir_contents("static", &target_dir);
    fs::copy(build_dir.join("troll-invasion.js"), target_dir.join("code.js")).unwrap();
    if target.as_ref().to_str().unwrap() == "wasm32" {
        let mut wasm_path = None;
        for entry in fs::read_dir(build_dir.join("deps")).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "wasm" {
                        assert!(wasm_path.is_none(), "Multiple .wasm files");
                        wasm_path = Some(entry.path());
                    }
                }
            }
        }
        let wasm_path = wasm_path.unwrap();
        fs::copy(&wasm_path, target_dir.join(wasm_path.file_name().unwrap())).unwrap();
    }
}