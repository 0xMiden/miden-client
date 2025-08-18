use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Only create archive in release mode to avoid slow debug builds
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    
    if profile == "release" {
        println!("cargo:rerun-if-changed=../../tests/src/");
        println!("cargo:rerun-if-changed=../../tests/Cargo.toml");
        
        // Create test archive during build
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
        let archive_path = Path::new(&out_dir).join("integration-tests.tar.zst");
        
        println!("Creating nextest archive at: {}", archive_path.display());
        
        // Check if nextest is available
        let nextest_available = Command::new("cargo")
            .args(["nextest", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        if nextest_available {
            println!("cargo:warning=Creating nextest archive - this may take a while...");
            
            // Create nextest archive
            let output = Command::new("cargo")
                .args([
                    "nextest",
                    "archive",
                    "--workspace",
                    "--exclude", "miden-client-web",
                    "--exclude", "testing-remote-prover",
                    "--exclude", "miden-integration-tests",  // Exclude the binary crate itself
                    "--release",
                    "--test=integration",
                    "--archive-file", archive_path.to_str().unwrap(),
                ])
                .current_dir("../..")  // Run from workspace root
                .output()
                .expect("Failed to execute cargo nextest archive");
                
            if !output.status.success() {
                eprintln!("nextest archive stderr: {}", String::from_utf8_lossy(&output.stderr));
                eprintln!("nextest archive stdout: {}", String::from_utf8_lossy(&output.stdout));
                println!("cargo:warning=Failed to create nextest archive - falling back to source mode");
                println!("cargo:rustc-env=HAS_EMBEDDED_ARCHIVE=false");
            } else {
                println!("cargo:warning=Nextest archive created successfully");
                println!("cargo:rustc-env=INTEGRATION_TESTS_ARCHIVE_PATH={}", archive_path.display());
                println!("cargo:rustc-env=HAS_EMBEDDED_ARCHIVE=true");
            }
        } else {
            println!("cargo:warning=nextest not available during build - archive not created");
            println!("cargo:rustc-env=HAS_EMBEDDED_ARCHIVE=false");
        }
    } else {
        println!("cargo:rustc-env=HAS_EMBEDDED_ARCHIVE=false");
    }
}