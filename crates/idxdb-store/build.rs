use std::process::Command;

/// Defines whether the build script should generate files in `/src`.
/// The docs.rs build pipeline has a read-only filesystem, so we have to avoid writing to `src`,
/// otherwise the docs will fail to build there. Note that writing to `OUT_DIR` is fine.
const CODEGEN: bool = option_env!("CODEGEN").is_some();

fn main() -> Result<(), String> {
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-env-changed=CODEGEN");
    if !CODEGEN {
        return Ok(());
    }

    // Install deps
    run_yarn(&[]).map_err(|e| format!("could not install ts dependencies: {e}"))?;

    // Build TS
    run_yarn(&["build"]).map_err(|e| format!("failed to build typescript: {e}"))?;

    Ok(())
}


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
