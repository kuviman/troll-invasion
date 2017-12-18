use std::process::Command;
use std::path::Path;

fn main() {
    #[cfg(not(windows))]
        assert!(Command::new("mvn").arg("package").current_dir("server-src").status().unwrap().success());
    #[cfg(windows)]
        assert!(Command::new("cmd").arg("/C").arg("mvn.cmd").arg("package").current_dir("server-src").status().unwrap().success());
    std::fs::copy(Path::new("server-src").join("target").join("troll-invasion.jar"), Path::new("target").join("troll-invasion.jar")).unwrap();
}