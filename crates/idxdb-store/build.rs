use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

struct BindingSpec {
    template: &'static str,
    js_file: &'static str,
    output: &'static str,
}

const JS_BINDINGS: &[BindingSpec] = &[
    BindingSpec {
        template: "build/js_bindings_templates/accounts.rs",
        js_file: "accounts.js",
        output: "account_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/auth.rs",
        js_file: "accounts.js",
        output: "auth_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/chain_data.rs",
        js_file: "chainData.js",
        output: "chain_data_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/export.rs",
        js_file: "export.js",
        output: "export_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/import.rs",
        js_file: "import.js",
        output: "import_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/notes.rs",
        js_file: "notes.js",
        output: "notes_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/settings.rs",
        js_file: "settings.js",
        output: "settings_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/sync.rs",
        js_file: "sync.js",
        output: "sync_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/transactions.rs",
        js_file: "transactions.js",
        output: "transactions_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/utils.rs",
        js_file: "utils.js",
        output: "utils_js_bindings.rs",
    },
    BindingSpec {
        template: "build/js_bindings_templates/schema.rs",
        js_file: "schema.js",
        output: "schema_js_bindings.rs",
    },
];

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
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|err| format!("missing CARGO_MANIFEST_DIR: {err}"))?;
    let src_dir = Path::new(&manifest_dir).join("src");
    let out_dir =
        PathBuf::from(env::var("OUT_DIR").map_err(|err| format!("missing OUT_DIR: {err}"))?);
    let js_out_dir = out_dir.join("js");
    let generated_bindings_dir = out_dir.join("generated_js_bindings");

    println!("cargo::rerun-if-changed={}", src_dir.join("package.json").display());
    println!("cargo::rerun-if-changed={}", src_dir.join("yarn.lock").display());
    println!("cargo::rerun-if-changed={}", src_dir.join("tsconfig.json").display());
    println!("cargo::rerun-if-changed={}", src_dir.join("tsconfig.bundler.json").display());
    println!("cargo::rerun-if-changed={}", src_dir.join("ts").display());
    println!("cargo::rerun-if-env-changed=MIDEN_IDXDB_STORE_PRESERVE_NODE_MODULES");

    if js_out_dir.exists() {
        fs::remove_dir_all(&js_out_dir).map_err(|err| {
            format!("could not clean js output dir {}: {err}", js_out_dir.display())
        })?;
    }
    fs::create_dir_all(&js_out_dir)
        .map_err(|err| format!("could not create js output dir {}: {err}", js_out_dir.display()))?;
    if generated_bindings_dir.exists() {
        fs::remove_dir_all(&generated_bindings_dir).map_err(|err| {
            format!(
                "could not clean generated bindings dir {}: {err}",
                generated_bindings_dir.display()
            )
        })?;
    }
    fs::create_dir_all(&generated_bindings_dir).map_err(|err| {
        format!(
            "could not create generated bindings dir {}: {err}",
            generated_bindings_dir.display()
        )
    })?;

    install_node_modules().map_err(|err| format!("could not install ts dependencies: {err}"))?;
    let result = build_typescript()
        .and_then(|_| copy_generated_artifacts(&src_dir, &js_out_dir))
        .and_then(|_| generate_js_bindings(&manifest_dir, &js_out_dir, &generated_bindings_dir))
        .map_err(|err| format!("failed to build typescript: {err}"));

    if let Err(err) = cleanup_typescript_side_effects(&src_dir) {
        eprintln!("cargo:warning=failed to clean temporary ts artifacts: {err}");
    }

    if !should_preserve_node_modules() {
        if let Err(err) = remove_node_modules() {
            eprintln!("cargo:warning=failed to remove node_modules directory: {err}");
        }
    }

    result
}

fn install_node_modules() -> Result<(), String> {
    let mut args = vec!["install", "--frozen-lockfile", "--silent"];
    if should_preserve_node_modules() {
        args.push("--check-files");
    }
    run_yarn(&args)
}

fn build_typescript() -> Result<(), String> {
    run_yarn(&["tsc", "--build", "--force", "./tsconfig.json"])
}

fn copy_generated_artifacts(src_dir: &Path, js_out_dir: &Path) -> Result<(), String> {
    let generated_dir = src_dir.join("js");
    if !generated_dir.exists() {
        return Err(format!("expected generated js artifacts at {}", generated_dir.display()));
    }

    for entry in fs::read_dir(&generated_dir)
        .map_err(|err| format!("failed to read {}: {err}", generated_dir.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read dir entry: {err}"))?;
        let file_type =
            entry.file_type().map_err(|err| format!("failed to read entry type: {err}"))?;
        if !file_type.is_file() {
            continue;
        }

        let destination = js_out_dir.join(entry.file_name());
        fs::copy(entry.path(), &destination).map_err(|err| {
            format!("failed to copy {} to {}: {err}", entry.path().display(), destination.display())
        })?;
    }

    Ok(())
}

fn generate_js_bindings(
    manifest_dir: &str,
    js_out_dir: &Path,
    generated_dir: &Path,
) -> Result<(), String> {
    for binding in JS_BINDINGS {
        let template_path = Path::new(manifest_dir).join(binding.template);
        println!("cargo::rerun-if-changed={}", template_path.display());

        let template = fs::read_to_string(&template_path)
            .map_err(|err| format!("failed to read template {}: {err}", template_path.display()))?;

        let js_path = js_out_dir.join(binding.js_file);
        let js_source = fs::read_to_string(&js_path)
            .map_err(|err| format!("failed to read generated js {}: {err}", js_path.display()))?;

        let inline_literal = format!("{js_source:?}");
        let rendered = template.replace("__INLINE_JS__", &inline_literal);
        let output_path = generated_dir.join(binding.output);
        fs::write(&output_path, rendered)
            .map_err(|err| format!("could not write {}: {err}", output_path.display()))?;
    }

    Ok(())
}

fn cleanup_typescript_side_effects(src_dir: &Path) -> Result<(), String> {
    let generated_dir = src_dir.join("js");
    if generated_dir.exists() {
        fs::remove_dir_all(&generated_dir)
            .map_err(|err| format!("failed to remove {}: {err}", generated_dir.display()))?;
    }

    let tsbuildinfo_path = src_dir.join("tsconfig.tsbuildinfo");
    if tsbuildinfo_path.exists() {
        fs::remove_file(&tsbuildinfo_path)
            .map_err(|err| format!("failed to remove {}: {err}", tsbuildinfo_path.display()))?;
    }

    Ok(())
}

fn should_preserve_node_modules() -> bool {
    env::var_os("MIDEN_IDXDB_STORE_PRESERVE_NODE_MODULES").is_some()
}

fn remove_node_modules() -> Result<(), String> {
    let node_modules_path = Path::new("src").join("node_modules");
    if node_modules_path.exists() {
        fs::remove_dir_all(&node_modules_path)
            .map_err(|err| format!("could not remove node_modules: {err}"))?;
    }
    Ok(())
}
