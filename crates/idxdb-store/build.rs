use std::process::Command;

fn main() -> miette::Result<(), String> {
    println!("cargo::rerun-if-changed=.");

    // Install dependencies for the TS project files with yarn
    Command::new("yarn").current_dir("src").status().map_err(|err| {
        format!("could not install ts dependencies -- have you installed yarn? got error: {err}")
    })?;

    // Build the TS files into JS and store the artifacts under src/js
    Command::new("yarn")
        .args(["build"])
        .current_dir("src")
        .status()
        .map_err(|err| format!("failed to run build command for typescript: {err}"))?;

    Ok(())
}
