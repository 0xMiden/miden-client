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

fn main() -> Result<(), String> {
    println!("cargo::rerun-if-changed=build.rs");

    if let Ok(code_gen_env_var) = std::env::var("CODEGEN")
        && code_gen_env_var == "1"
    {
        println!("cargo::warning=Building JS files");

        // Install deps
        run_yarn(&[]).map_err(|e| format!("could not install ts dependencies: {e}"))?;

        // Build TS
        run_yarn(&["build"]).map_err(|e| format!("failed to build typescript: {e}"))?;

        // Remove files that don't have js extension.
        for artifact in std::fs::read_dir("./src/js").expect("js folder should exist") {
            if !artifact.as_ref().unwrap().file_name().into_string().unwrap().ends_with(".js") {
                std::fs::remove_file(artifact.unwrap().path()).expect("could not delete artifact");
            }
        }
    }
    Ok(())
}
