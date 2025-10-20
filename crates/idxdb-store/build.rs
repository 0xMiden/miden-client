use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=src/ts");
    Command::new("yarn")
        .args(&["build"])
        .current_dir("./src")
        .status()
        .expect("failed to build typescript files");
}
