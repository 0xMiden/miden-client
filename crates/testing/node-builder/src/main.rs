#![recursion_limit = "256"]

use std::{fs, io::Write, path::PathBuf, process, time::Duration};

use clap::Parser;
use node_builder::{DEFAULT_BATCH_INTERVAL, DEFAULT_BLOCK_INTERVAL, DEFAULT_RPC_PORT, NodeBuilder};

const PID_FILE: &str = "miden-node.pid";

#[derive(Parser, Debug)]
#[command(name = "node-builder", about = "Testing node builder for Miden")]
struct Args {
    /// Path to genesis configuration file
    #[arg(long)]
    genesis_config: Option<PathBuf>,
}

fn write_pid_file() -> anyhow::Result<()> {
    let pid = process::id();
    let mut file = fs::File::create(PID_FILE)?;
    file.write_all(pid.to_string().as_bytes())?;
    Ok(())
}

fn ensure_data_dir(data_dir: &PathBuf) -> anyhow::Result<()> {
    if data_dir.exists() {
        // Remove all contents of the directory
        fs::remove_dir_all(data_dir)?;
        println!("Cleaned existing data directory at {}", data_dir.display());
    }
    // Create fresh directory
    fs::create_dir_all(data_dir)?;

    // Set permissions to allow read/write
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(data_dir)?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(data_dir, perms)?;
    }

    println!("Created data directory at {}", data_dir.display());
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let data_dir = PathBuf::from("./data");
    ensure_data_dir(&data_dir)?;
    write_pid_file()?;

    let mut builder = NodeBuilder::new(data_dir)
        .with_rpc_port(DEFAULT_RPC_PORT)
        .with_block_interval(Duration::from_millis(DEFAULT_BLOCK_INTERVAL))
        .with_batch_interval(Duration::from_millis(DEFAULT_BATCH_INTERVAL));

    if let Some(genesis_config) = args.genesis_config {
        println!("Using genesis configuration from: {}", genesis_config.display());
        builder = builder.with_genesis_config(genesis_config);
    }

    let handle = builder.start().await?;
    println!("Node started successfully with PID: {}", process::id());

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    handle.stop().await?;
    println!("Node stopped successfully");

    Ok(())
}
