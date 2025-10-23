use std::process::Command;

fn main() -> miette::Result<(), String> {
    println!("cargo::rerun-if-changed=.");

    // Install dependencies for the TS project files with yarn
    let status = Command::new("yarn").current_dir("src").status().map_err(|err| {
        format!("could not install ts dependencies -- have you installed yarn? got error: {err}")
    })?;
    if !status.success() {
        return Err(format!("could not install ts dependencies: yarn exited with status {status}"));
    }

    // Build the TS files into JS and store the artifacts under src/js
    let status = Command::new("yarn")
        .args(["build"])
        .current_dir("src")
        .status()
        .map_err(|err| format!("failed to run build command for typescript: {err}"))?;
    if !status.success() {
        return Err(format!("failed to build typescript: yarn build exited with status {status}"));
    }

    Ok(())
}
