use std::process::Command;

#[cfg(windows)]
fn run_yarn(args: &[&str]) -> Result<(), String> {
    let status = Command::new("cmd")
        .args(["/C", "yarn"])
        .args(args)
        .current_dir(&std::env::var("OUT_DIR").unwrap())
        .status()
        .map_err(|err| format!("could not run yarn via cmd: {err}"))?;
    if !status.success() {
        return Err(format!("yarn exited with status {status}"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn run_yarn(args: &[&str]) -> Result<(), String> {
    use std::{path::PathBuf, str::FromStr};

    let mut path = PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();
    path.push("src");
    let status = Command::new("yarn")
        .args(args)
        .current_dir(path)
        .status()
        .map_err(|err| format!("could not run yarn: {err}"))?;
    if !status.success() {
        return Err(format!("yarn exited with status {status}"));
    }
    Ok(())
}

fn main() -> miette::Result<(), String> {
    // println!("cargo::rerun-if-changed=src");

    Command::new("mkdir")
        .args([&std::env::var("OUT_DIR").unwrap()])
        .status()
        .unwrap();

    Command::new("cp")
        .args(["-r", "src", &std::env::var("OUT_DIR").unwrap()])
        .status()
        .unwrap();

    // Install deps
    run_yarn(&[]).map_err(|e| format!("could not install ts dependencies: {e}"))?;

    // Build TS
    run_yarn(&["build"]).map_err(|e| format!("failed to build typescript: {e}"))?;

    // Command::new("cp")
    //     .args(["-r", &format!("{}/src/js", &std::env::var("OUT_DIR").unwrap()), "."])
    //     .status()
    //     .unwrap();

    Ok(())
}
