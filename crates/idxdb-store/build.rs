use std::{path::Path, process::Command};

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
    println!("cargo::rerun-if-changed=src/ts");
    println!("cargo::rerun-if-changed=src/package.json");
    println!("cargo::rerun-if-changed=src/tsconfig.json");

    // Check if JS files already exist (e.g., during cargo package verification)
    let js_dir = Path::new("src/js");
    if js_dir.exists() && js_dir.is_dir() {
        // Check if at least one expected JS file exists
        if Path::new("src/js/accounts.js").exists() {
            println!("cargo::warning=JS files already exist, skipping TypeScript compilation");
            return Ok(());
        }
    }

    // Install deps
    run_yarn(&[]).map_err(|e| format!("could not install ts dependencies: {e}"))?;

    // Build TS
    run_yarn(&["build"]).map_err(|e| format!("failed to build typescript: {e}"))?;

    Ok(())
}
