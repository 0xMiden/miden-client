use std::process::Command;

#[cfg(windows)]
fn run_yarn(args: &[&str]) -> Result<(), String> {
    let status = Command::new("cmd")
        .args(["/C", "yarn"])
        .args(args)
        .current_dir("src")
        .status()
        .map_err(|err| format!("could not run yarn via cmd: {err}"))?;
    if !status.success() {
        return Err(format!("yarn exited with status {status}"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn run_yarn(args: &[&str]) -> Result<(), String> {
    let status = Command::new("yarn")
        .args(args)
        .current_dir("src")
        .status()
        .map_err(|err| format!("could not run yarn: {err}"))?;
    if !status.success() {
        return Err(format!("yarn exited with status {status}"));
    }
    Ok(())
}

fn main() -> miette::Result<(), String> {
    println!("cargo::rerun-if-changed=src");

    // Install deps
    run_yarn(&[]).map_err(|e| format!("could not install ts dependencies: {e}"))?;

    // Build TS
    run_yarn(&["build"]).map_err(|e| format!("failed to build typescript: {e}"))?;

    Ok(())
}
