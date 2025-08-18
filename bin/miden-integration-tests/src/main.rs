use std::env;
use std::process::Command;
use clap::Parser;
use serde::{Deserialize, Serialize};
use camino::Utf8Path;

// nextest-runner imports
use nextest_runner::{
    reuse_build::{ReuseBuildInfo, ArchiveFormat, ExtractDestination},
};

#[derive(Parser, Debug)]
#[command(name = "miden-integration-tests")]
#[command(about = "Run Miden integration tests with configurable parameters")]
#[command(long_about = "Run Miden integration tests with configurable RPC parameters.

This tool can run in different modes:
1. Archive mode (release builds): Uses pre-built test archive with nextest-runner
2. Source mode (debug builds): Compiles and runs tests from source code with nextest

The tool will automatically detect the best available method.
Archive mode provides fastest execution as tests are pre-compiled.")]
struct Args {
    /// RPC endpoint host (e.g., localhost, rpc.devnet.miden.io)
    #[arg(long, default_value = "localhost")]
    host: String,

    /// RPC endpoint port
    #[arg(long)]
    port: Option<u16>,

    /// RPC endpoint protocol (http or https)
    #[arg(long, default_value = "http")]
    protocol: String,

    /// Timeout in milliseconds for RPC requests
    #[arg(long, default_value = "10000")]
    timeout: u64,

    /// Run ignored tests only
    #[arg(long)]
    run_ignored: bool,

    /// Additional test patterns to match specific tests
    test_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcConfig {
    endpoint: EndpointConfig,
    timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EndpointConfig {
    host: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
    protocol: String,
}

fn has_embedded_archive() -> bool {
    option_env!("HAS_EMBEDDED_ARCHIVE").unwrap_or("false") == "true"
}

fn run_with_archive(archive_path: &str, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Running test archive with nextest-runner 0.85.0");
    
    // Convert to Utf8Path for nextest-runner
    let archive_utf8_path: &Utf8Path = archive_path.try_into()?;
    
    // Extract the archive using nextest-runner programmatically for better control
    println!("📦 Extracting archive with nextest-runner...");
    let reuse_build_info = ReuseBuildInfo::extract_archive(
        archive_utf8_path,
        ArchiveFormat::TarZst,
        ExtractDestination::TempDir { persist: false },
        |event| {
            println!("Extraction progress: {:?}", event);
            Ok(())
        },
        None, // No workspace remapping
    )?;
    
    println!("✅ Archive extracted successfully");
    
    // Show available information from ReuseBuildInfo
    println!("📂 Archive extracted successfully");
    if let Some(_binaries_metadata) = &reuse_build_info.binaries_metadata {
        println!("📦 Found binaries metadata");
    }
    if let Some(_cargo_metadata) = &reuse_build_info.cargo_metadata {
        println!("📦 Found cargo metadata");
    }
    
    // Get the list of test binaries from the extracted archive
    if let Some(_binaries_metadata) = &reuse_build_info.binaries_metadata {
        println!("🔍 Found binaries metadata in archive");
        
        // For nextest-runner 0.85.0, use the CLI approach for reliability
        // since the extraction details are not easily accessible
        let mut cmd = Command::new("cargo");
        cmd.arg("nextest")
            .arg("run")
            .arg("--archive-file")
            .arg(archive_path);
        
        if args.run_ignored {
            cmd.arg("--run-ignored").arg("ignored-only");
        }

        // Add any test patterns
        if !args.test_patterns.is_empty() {
            cmd.args(&args.test_patterns);
        }

        println!("Running extracted tests: {:?}", cmd);

        // Execute the command
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    } else {
        // Fallback to direct archive file usage
        println!("⚠️  No binaries metadata found, falling back to direct archive usage");
        
        let mut cmd = Command::new("cargo");
        cmd.arg("nextest")
            .arg("run")
            .arg("--archive-file")
            .arg(archive_path);

        if args.run_ignored {
            cmd.arg("--run-ignored").arg("ignored-only");
        }

        // Add any test patterns
        if !args.test_patterns.is_empty() {
            cmd.args(&args.test_patterns);
        }

        println!("Running: {:?}", cmd);

        // Execute the command
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create temporary RPC config
    let config = RpcConfig {
        endpoint: EndpointConfig {
            host: args.host.clone(),
            port: args.port,
            protocol: args.protocol.clone(),
        },
        timeout: args.timeout,
    };

    let config_content = toml::to_string(&config)?;
    
    // Set environment variable for the config content
    // This will be read by the test utilities instead of the hardcoded file
    // SAFETY: We're setting an environment variable that will be read by our own test process.
    // This is safe because we control both the setting and reading of this variable.
    unsafe {
        env::set_var("MIDEN_CLIENT_RPC_CONFIG_OVERRIDE", config_content);
    }

    // Check if we have an embedded archive (from release build)
    let has_embedded = has_embedded_archive();
    
    if has_embedded {
        println!("🚀 Using embedded test archive (nextest CLI mode)");
        
        // Get the embedded archive path and run with nextest CLI
        let archive_path = option_env!("INTEGRATION_TESTS_ARCHIVE_PATH")
            .ok_or("No embedded archive available")?;
        return run_with_archive(archive_path, &args);
    } else {
        println!("🔧 Using source mode with nextest");
        
        // Run tests from source using nextest CLI
        let mut cmd = Command::new("cargo");
        cmd.arg("nextest")
            .arg("run")
            .arg("--workspace")
            .arg("--exclude")
            .arg("miden-client-web")
            .arg("--exclude")
            .arg("testing-remote-prover")
            .arg("--exclude")
            .arg("miden-integration-tests")  // Exclude the binary crate itself
            .arg("--release")
            .arg("--test=integration");

        if args.run_ignored {
            cmd.arg("--run-ignored").arg("ignored-only");
        }

        // Add any test patterns
        if !args.test_patterns.is_empty() {
            cmd.args(&args.test_patterns);
        }

        println!("Running: {:?}", cmd);

        // Execute the command
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}