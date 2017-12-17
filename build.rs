use std::process::Command;
use std::path::Path;

fn main() {
    assert!(Command::new("mvn").arg("package").current_dir("server-src").status().unwrap().success());
    std::fs::copy(Path::new("server-src").join("target").join("troll-invasion.jar"), Path::new("target").join("troll-invasion.jar")).unwrap();
}